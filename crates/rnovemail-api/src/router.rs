use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use axum::{Json, Router, http::HeaderMap, response::Redirect, routing::get};
use chrono::Utc;
use rnovemail_auth::verify_login_secret;
use rnovemail_domain::{
    AuditActor, AuditEvent, AuditResult, DomainName, EmailAddress, InboundMessage,
    InboundMessageDetail, Mailbox, MailboxId, MailboxStatus, MessageStatus, MessageTimelineEntry,
    OutboundMessage, ProviderAccount, ProviderAccountId, ProviderType, User, UserRole, UserStatus,
};
use rnovemail_providers::{
    MailProvider, ProviderError, ProviderEvent, ProviderSendReceipt, ProviderWebhookRequest,
    ResendProvider, SendMailRequest,
};
use rnovemail_store::{AppStore, StoreError};
use rnovemail_webhook::SignatureVerifier;
use secrecy::{ExposeSecret, SecretString};
use subtle::ConstantTimeEq;

use crate::session::{SessionError, SessionPrincipal, SessionRegistry, SessionRole};
use crate::{
    admin_pages, admin_routes, mail_routes, middleware::ApiRejection, openapi, session_routes,
    user_pages, webhook_routes,
};

pub(crate) const SESSION_COOKIE: &str = "rnovemail_session";

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    ready: bool,
    admin_token: Option<String>,
    store: Option<Arc<dyn AppStore>>,
    data: RwLock<AppData>,
    public_base_url: RwLock<Option<String>>,
    resend_endpoint: RwLock<String>,
    sessions: RwLock<SessionRegistry>,
}

#[derive(Default)]
struct AppData {
    users: HashMap<String, User>,
    domains: HashSet<DomainName>,
    providers: Vec<ProviderAccount>,
    mailboxes: HashMap<String, Mailbox>,
    outbound_messages: HashMap<String, OutboundMessage>,
    inbound_messages: HashMap<String, InboundMessage>,
    webhook_bindings: HashMap<String, WebhookBinding>,
    webhook_events: HashSet<String>,
    audit_events: Vec<AuditEvent>,
}

#[derive(Clone)]
struct WebhookBinding {
    provider_type: ProviderType,
    secret: SecretString,
}

struct InboundRecord {
    provider_account_id: ProviderAccountId,
    provider_event_id: String,
    from: EmailAddress,
    recipients: Vec<EmailAddress>,
    subject: String,
    text: String,
    detail: Option<InboundMessageDetail>,
}

impl AppState {
    pub fn empty() -> Self {
        Self::new(None)
    }

    pub fn with_admin_token(token: impl Into<String>) -> Self {
        Self::new(Some(token.into()))
    }

    pub fn with_resend_endpoint(self, endpoint: impl Into<String>) -> Self {
        if let Ok(mut value) = self.inner.resend_endpoint.write() {
            *value = endpoint.into();
        }
        self
    }

    pub fn with_public_base_url(self, base_url: impl Into<String>) -> Self {
        if let Ok(mut value) = self.inner.public_base_url.write() {
            *value = normalize_public_base_url(base_url.into());
        }
        self
    }

    pub async fn with_persistent_store<S>(
        admin_token: Option<String>,
        store: Arc<S>,
    ) -> Result<Self, StoreError>
    where
        S: AppStore + 'static,
    {
        let data = load_app_data(store.as_ref()).await?;
        let store: Arc<dyn AppStore> = store;
        Ok(Self::from_data(admin_token, Some(store), data))
    }

    pub fn ready(&self) -> bool {
        self.inner.ready
    }

    pub(crate) fn require_admin(&self, headers: &HeaderMap) -> Result<(), ApiRejection> {
        match optional_bearer_token(headers)? {
            Some(token) => self.verify_admin_token(token),
            None => self.admin_principal(headers).map(|_| ()),
        }
    }

    pub(crate) fn admin_principal(
        &self,
        headers: &HeaderMap,
    ) -> Result<SessionPrincipal, ApiRejection> {
        self.session_principal(headers, SessionRole::Admin)
    }

    pub(crate) fn user_principal(
        &self,
        headers: &HeaderMap,
    ) -> Result<SessionPrincipal, ApiRejection> {
        self.session_principal(headers, SessionRole::User)
    }

    pub(crate) fn create_session(
        &self,
        role: SessionRole,
        subject: String,
        headers: &HeaderMap,
    ) -> Result<String, ApiRejection> {
        self.write_sessions()
            .map(|mut sessions| sessions.create(role, subject, request_fingerprint(headers)))
    }

    pub(crate) fn remove_session(&self, headers: &HeaderMap) -> Result<(), ApiRejection> {
        let Some(session_id) = session_cookie(headers) else {
            return Ok(());
        };
        self.write_sessions()?.remove(session_id);
        Ok(())
    }

    pub(crate) fn ensure_login_allowed(&self, key: &str) -> Result<(), ApiRejection> {
        match self.write_sessions()?.ensure_login_allowed(key) {
            Ok(()) => Ok(()),
            Err(SessionError::Locked) => Err(ApiRejection::TooManyLoginAttempts),
            Err(_) => Err(ApiRejection::InvalidApiToken),
        }
    }

    pub(crate) fn record_login_failure(&self, key: String) -> Result<(), ApiRejection> {
        self.write_sessions()?.record_failure(key);
        Ok(())
    }

    pub(crate) fn record_login_success(&self, key: &str) -> Result<(), ApiRejection> {
        self.write_sessions()?.record_success(key);
        Ok(())
    }

    pub(crate) fn verify_user_login(
        &self,
        email: &EmailAddress,
        presented: &str,
    ) -> Result<User, ApiRejection> {
        let user = self.user_by_email(email)?;
        ensure_active_user(&user)?;
        let Some(hash) = user.login_secret_hash() else {
            return Err(ApiRejection::InvalidApiToken);
        };
        verify_login_secret(hash, presented).map_err(|_| ApiRejection::InvalidApiToken)?;
        Ok(user)
    }

    pub(crate) fn list_users(&self) -> Result<Vec<User>, ApiRejection> {
        let data = self.read_data()?;
        Ok(data.users.values().cloned().collect())
    }

    pub(crate) fn list_domains(&self) -> Result<Vec<DomainName>, ApiRejection> {
        let data = self.read_data()?;
        Ok(data.domains.iter().cloned().collect())
    }

    pub(crate) fn list_providers(&self) -> Result<Vec<ProviderAccount>, ApiRejection> {
        let data = self.read_data()?;
        Ok(data.providers.clone())
    }

    pub(crate) fn public_base_url(&self) -> Result<Option<String>, ApiRejection> {
        self.inner
            .public_base_url
            .read()
            .map(|value| value.clone())
            .map_err(|_| ApiRejection::StateUnavailable)
    }

    pub(crate) fn list_mailboxes(&self) -> Result<Vec<Mailbox>, ApiRejection> {
        let data = self.read_data()?;
        Ok(data.mailboxes.values().cloned().collect())
    }

    pub(crate) fn list_outbound_messages(&self) -> Result<Vec<OutboundMessage>, ApiRejection> {
        let data = self.read_data()?;
        Ok(data.outbound_messages.values().cloned().collect())
    }

    pub(crate) fn list_inbound_messages(&self) -> Result<Vec<InboundMessage>, ApiRejection> {
        let data = self.read_data()?;
        Ok(data.inbound_messages.values().cloned().collect())
    }

    pub(crate) fn inbound_message_by_id(&self, id: &str) -> Result<InboundMessage, ApiRejection> {
        let data = self.read_data()?;
        data.inbound_messages
            .get(id)
            .cloned()
            .ok_or(ApiRejection::NotFound)
    }

    pub(crate) async fn hydrate_inbound_message_detail(
        &self,
        message: InboundMessage,
    ) -> InboundMessage {
        match self.missing_inbound_detail(&message).await {
            Ok(Some(detail)) => self.inbound_message_with_detail(message, detail).await,
            _ => message,
        }
    }

    pub(crate) fn list_audit(&self) -> Result<Vec<AuditEvent>, ApiRejection> {
        let data = self.read_data()?;
        Ok(data.audit_events.clone())
    }

    pub(crate) async fn add_user_with_secret(
        &self,
        display_name: String,
        email: EmailAddress,
        roles: Vec<UserRole>,
        login_secret_hash: Option<String>,
    ) -> Result<User, ApiRejection> {
        let user = User::assign(display_name, email.clone(), roles)
            .with_login_secret_hash(login_secret_hash);
        self.persist_user(&user).await?;
        self.insert_user(user.clone())?;
        self.record_audit("admin.user.create", email.as_str())
            .await?;
        Ok(user)
    }

    pub(crate) async fn update_user(
        &self,
        email: &EmailAddress,
        patch: UserPatch,
    ) -> Result<User, ApiRejection> {
        let mut user = self.user_by_email(email)?;
        patch.apply(&mut user);
        self.persist_user(&user).await?;
        self.insert_user(user.clone())?;
        self.record_audit("admin.user.update", email.as_str())
            .await?;
        Ok(user)
    }

    fn insert_user(&self, user: User) -> Result<(), ApiRejection> {
        let mut data = self.write_data()?;
        data.users
            .insert(user.primary_email().as_str().to_string(), user);
        Ok(())
    }

    async fn persist_user(&self, user: &User) -> Result<(), ApiRejection> {
        if let Some(store) = self.store() {
            store.put_user(user.clone()).await.map_err(store_error)?;
        }
        Ok(())
    }

    pub(crate) async fn add_domain(&self, domain: DomainName) -> Result<DomainName, ApiRejection> {
        if let Some(store) = self.store() {
            store
                .put_domain(domain.clone())
                .await
                .map_err(store_error)?;
        }
        {
            let mut data = self.write_data()?;
            data.domains.insert(domain.clone());
        }
        self.record_audit("admin.domain.create", domain.as_str())
            .await?;
        Ok(domain)
    }

    pub(crate) async fn update_domain(
        &self,
        old_domain: DomainName,
        new_domain: DomainName,
    ) -> Result<DomainName, ApiRejection> {
        if let Some(store) = self.store() {
            store
                .delete_domain(&old_domain)
                .await
                .map_err(store_error)?;
            store
                .put_domain(new_domain.clone())
                .await
                .map_err(store_error)?;
        }
        {
            let mut data = self.write_data()?;
            data.domains.remove(&old_domain);
            data.domains.insert(new_domain.clone());
        }
        self.record_audit("admin.domain.update", new_domain.as_str())
            .await?;
        Ok(new_domain)
    }

    pub(crate) async fn delete_domain(&self, domain: DomainName) -> Result<(), ApiRejection> {
        if let Some(store) = self.store() {
            store.delete_domain(&domain).await.map_err(store_error)?;
        }
        {
            let mut data = self.write_data()?;
            data.domains.remove(&domain);
        }
        self.record_audit("admin.domain.delete", domain.as_str())
            .await
    }

    pub(crate) async fn add_provider(
        &self,
        provider: ProviderAccount,
    ) -> Result<ProviderAccount, ApiRejection> {
        let binding = provider_webhook_binding(&provider)?;
        if let Some(store) = self.store() {
            store
                .put_provider(provider.clone())
                .await
                .map_err(store_error)?;
        }
        {
            let mut data = self.write_data()?;
            insert_webhook_binding(&mut data, &provider, binding);
            data.providers.push(provider.clone());
        }
        self.record_audit("admin.provider.create", &provider_key(&provider))
            .await?;
        Ok(provider)
    }

    pub(crate) async fn update_provider(
        &self,
        id: &str,
        patch: ProviderPatch,
    ) -> Result<ProviderAccount, ApiRejection> {
        let mut provider = self.provider_by_id(id)?;
        patch.apply(&mut provider);
        let binding = provider_webhook_binding(&provider)?;
        if let Some(store) = self.store() {
            store
                .put_provider(provider.clone())
                .await
                .map_err(store_error)?;
        }
        self.replace_provider(provider.clone(), binding)?;
        self.record_audit("admin.provider.update", id).await?;
        Ok(provider)
    }

    pub(crate) async fn delete_provider(&self, id: &str) -> Result<(), ApiRejection> {
        let provider = self.provider_by_id(id)?;
        if let Some(store) = self.store() {
            store
                .delete_provider(&provider)
                .await
                .map_err(store_error)?;
        }
        self.remove_provider(id)?;
        self.record_audit("admin.provider.delete", id).await
    }

    pub(crate) fn provider_for_sender(
        &self,
        sender: &EmailAddress,
    ) -> Result<ProviderAccount, ApiRejection> {
        let data = self.read_data()?;
        if !data.domains.contains(sender.domain()) {
            return Err(ApiRejection::NoProviderForDomain);
        }
        data.providers
            .iter()
            .find(|provider| provider.serves_domain(sender.domain()))
            .cloned()
            .ok_or(ApiRejection::NoProviderForDomain)
    }

    pub(crate) async fn send_mail(
        &self,
        request: SendMailRequest,
    ) -> Result<(ProviderAccount, ProviderSendReceipt), ApiRejection> {
        let provider = self.provider_for_sender(request.from())?;
        let snapshot = request.clone();
        let receipt = self.deliver_with_provider(&provider, request).await?;
        self.record_outbound_message(&provider, &snapshot, &receipt)
            .await?;
        self.record_audit("mail.outbound.send", &provider_key(&provider))
            .await?;
        Ok((provider, receipt))
    }

    pub(crate) async fn send_user_mail(
        &self,
        user_email: &str,
        request: SendMailRequest,
    ) -> Result<(ProviderAccount, ProviderSendReceipt), ApiRejection> {
        let email = EmailAddress::parse(user_email).map_err(|_| ApiRejection::BadRequest)?;
        self.ensure_user_can_send(&email, request.from())?;
        self.send_mail(request).await
    }

    async fn record_outbound_message(
        &self,
        provider: &ProviderAccount,
        request: &SendMailRequest,
        receipt: &ProviderSendReceipt,
    ) -> Result<(), ApiRejection> {
        let message = outbound_message(provider, request, receipt);
        self.persist_outbound_message(&message).await?;
        self.insert_outbound_message(message)
    }

    async fn persist_outbound_message(
        &self,
        message: &OutboundMessage,
    ) -> Result<(), ApiRejection> {
        if let Some(store) = self.store() {
            store
                .put_outbound(message.clone())
                .await
                .map_err(store_error)?;
        }
        Ok(())
    }

    fn insert_outbound_message(&self, message: OutboundMessage) -> Result<(), ApiRejection> {
        let mut data = self.write_data()?;
        data.outbound_messages
            .insert(serialized_key(&message.id), message);
        Ok(())
    }

    pub(crate) async fn add_mailbox(
        &self,
        owner_email: EmailAddress,
        mailbox_email: EmailAddress,
    ) -> Result<Mailbox, ApiRejection> {
        let owner_id = self.owner_id(&owner_email)?;
        let mailbox = Mailbox::assign(owner_id, mailbox_email.clone());
        if let Some(store) = self.store() {
            store
                .put_mailbox(mailbox.clone())
                .await
                .map_err(store_error)?;
        }
        {
            let mut data = self.write_data()?;
            data.mailboxes
                .insert(mailbox_email.as_str().to_string(), mailbox.clone());
        }
        self.record_audit("admin.mailbox.create", mailbox_email.as_str())
            .await?;
        Ok(mailbox)
    }

    pub(crate) async fn update_mailbox(
        &self,
        email: &EmailAddress,
        patch: MailboxPatch,
    ) -> Result<Mailbox, ApiRejection> {
        let mut mailbox = self.mailbox_by_email(email)?;
        patch.apply(&mut mailbox);
        if let Some(store) = self.store() {
            store
                .put_mailbox(mailbox.clone())
                .await
                .map_err(store_error)?;
        }
        self.insert_mailbox(mailbox.clone())?;
        self.record_audit("admin.mailbox.update", email.as_str())
            .await?;
        Ok(mailbox)
    }

    pub(crate) async fn ingest_webhook(
        &self,
        provider: &str,
        account_id: &str,
        request: ProviderWebhookRequest,
    ) -> Result<(), ApiRejection> {
        let account = self.provider_by_id(account_id)?;
        let binding = self.webhook_binding(account_id)?;
        ensure_provider_matches(provider, binding.provider_type)?;
        let events = verify_webhook(binding, request)?;
        self.remember_webhook_events(provider, &account, events)
            .await
    }

    fn new(admin_token: Option<String>) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                ready: true,
                admin_token,
                store: None,
                data: RwLock::new(AppData::default()),
                public_base_url: RwLock::new(None),
                resend_endpoint: RwLock::new(default_resend_endpoint()),
                sessions: RwLock::new(SessionRegistry::default()),
            }),
        }
    }

    fn from_data(
        admin_token: Option<String>,
        store: Option<Arc<dyn AppStore>>,
        data: AppData,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                ready: true,
                admin_token,
                store,
                data: RwLock::new(data),
                public_base_url: RwLock::new(None),
                resend_endpoint: RwLock::new(default_resend_endpoint()),
                sessions: RwLock::new(SessionRegistry::default()),
            }),
        }
    }

    fn store(&self) -> Option<Arc<dyn AppStore>> {
        self.inner.store.clone()
    }

    fn webhook_binding(&self, account_id: &str) -> Result<WebhookBinding, ApiRejection> {
        let data = self.read_data()?;
        data.webhook_bindings
            .get(account_id)
            .cloned()
            .ok_or(ApiRejection::InvalidWebhookSignature)
    }

    async fn remember_webhook_events(
        &self,
        provider: &str,
        account: &ProviderAccount,
        events: Vec<ProviderEvent>,
    ) -> Result<(), ApiRejection> {
        for event in events {
            self.remember_provider_event(provider, account, event)
                .await?;
        }
        Ok(())
    }

    async fn remember_provider_event(
        &self,
        provider: &str,
        account: &ProviderAccount,
        event: ProviderEvent,
    ) -> Result<(), ApiRejection> {
        let event_id = provider_event_id(&event).to_string();
        self.remember_webhook_event(provider, &event_id).await?;
        self.apply_provider_event(account, event).await
    }

    async fn apply_provider_event(
        &self,
        account: &ProviderAccount,
        event: ProviderEvent,
    ) -> Result<(), ApiRejection> {
        match event {
            ProviderEvent::Inbound {
                provider_event_id,
                from,
                to,
                subject,
                text,
            } => {
                let detail = self
                    .inbound_message_detail(account, &provider_event_id)
                    .await;
                self.record_inbound_messages(InboundRecord {
                    provider_account_id: account.id(),
                    provider_event_id,
                    from,
                    recipients: to,
                    subject,
                    text,
                    detail,
                })
                .await
            }
            ProviderEvent::Delivered { .. }
            | ProviderEvent::Bounced { .. }
            | ProviderEvent::Complained { .. } => self.record_delivery_event(account).await,
        }
    }

    async fn record_delivery_event(&self, _account: &ProviderAccount) -> Result<(), ApiRejection> {
        Ok(())
    }

    async fn record_inbound_messages(&self, record: InboundRecord) -> Result<(), ApiRejection> {
        for recipient in &record.recipients {
            self.record_inbound_for_recipient(&record, recipient)
                .await?;
        }
        Ok(())
    }

    async fn record_inbound_for_recipient(
        &self,
        record: &InboundRecord,
        recipient: &EmailAddress,
    ) -> Result<(), ApiRejection> {
        let Some(mailbox) = self.optional_mailbox_by_email(recipient)? else {
            return Ok(());
        };
        if !is_inbound_mailbox(&mailbox) {
            return Ok(());
        }
        let message = inbound_message(
            record.provider_account_id,
            &record.provider_event_id,
            &record.from,
            &mailbox,
            &record.subject,
            &record.text,
            record.detail.clone(),
        );
        self.persist_inbound_message(&message).await?;
        self.insert_inbound_message(message)
    }

    async fn inbound_message_detail(
        &self,
        account: &ProviderAccount,
        provider_event_id: &str,
    ) -> Option<InboundMessageDetail> {
        self.retrieve_inbound_message_detail(account, provider_event_id)
            .await
            .ok()
    }

    async fn retrieve_inbound_message_detail(
        &self,
        account: &ProviderAccount,
        provider_event_id: &str,
    ) -> Result<InboundMessageDetail, ApiRejection> {
        match account.provider_type() {
            ProviderType::Resend => {
                self.retrieve_resend_inbound_detail(account, provider_event_id)
                    .await
            }
        }
    }

    async fn retrieve_resend_inbound_detail(
        &self,
        account: &ProviderAccount,
        provider_event_id: &str,
    ) -> Result<InboundMessageDetail, ApiRejection> {
        let api_key = account
            .api_key()
            .ok_or(ApiRejection::ProviderApiKeyMissing)?;
        let endpoint = self.resend_endpoint()?;
        ResendProvider::with_endpoint(SecretString::new(api_key.to_string()), endpoint)
            .retrieve_received_email(provider_event_id)
            .await
            .map_err(provider_error)
    }

    async fn missing_inbound_detail(
        &self,
        message: &InboundMessage,
    ) -> Result<Option<InboundMessageDetail>, ApiRejection> {
        if message.detail.is_some() {
            return Ok(None);
        }
        let account = self.provider_for_inbound_message(message)?;
        self.retrieve_inbound_message_detail(&account, &message.provider_event_id)
            .await
            .map(Some)
    }

    async fn inbound_message_with_detail(
        &self,
        mut message: InboundMessage,
        detail: InboundMessageDetail,
    ) -> InboundMessage {
        message.subject = inbound_subject_text(&message.subject, Some(&detail));
        message.text = inbound_body_text(&message.text, Some(&detail));
        message.detail = Some(detail);
        let _ = self.persist_inbound_message(&message).await;
        let _ = self.insert_inbound_message(message.clone());
        message
    }

    async fn persist_inbound_message(&self, message: &InboundMessage) -> Result<(), ApiRejection> {
        if let Some(store) = self.store() {
            store
                .put_inbound(message.clone())
                .await
                .map_err(store_error)?;
        }
        Ok(())
    }

    fn insert_inbound_message(&self, message: InboundMessage) -> Result<(), ApiRejection> {
        let mut data = self.write_data()?;
        data.inbound_messages
            .insert(serialized_key(&message.id), message);
        Ok(())
    }

    async fn remember_webhook_event(
        &self,
        provider: &str,
        event_id: &str,
    ) -> Result<(), ApiRejection> {
        match self.store() {
            Some(store) => remember_persistent_event(store, provider, event_id).await,
            None => self.remember_memory_event(provider, event_id),
        }
    }

    fn remember_memory_event(&self, provider: &str, event_id: &str) -> Result<(), ApiRejection> {
        let mut data = self.write_data()?;
        match data
            .webhook_events
            .insert(webhook_event_key(provider, event_id))
        {
            true => Ok(()),
            false => Err(ApiRejection::DuplicateWebhookEvent),
        }
    }

    pub(crate) fn verify_admin_token(&self, presented: &str) -> Result<(), ApiRejection> {
        let Some(expected) = &self.inner.admin_token else {
            return Err(ApiRejection::InvalidApiToken);
        };
        match bool::from(expected.as_bytes().ct_eq(presented.as_bytes())) {
            true => Ok(()),
            false => Err(ApiRejection::InvalidApiToken),
        }
    }

    fn session_principal(
        &self,
        headers: &HeaderMap,
        role: SessionRole,
    ) -> Result<SessionPrincipal, ApiRejection> {
        let Some(session_id) = session_cookie(headers) else {
            return Err(ApiRejection::MissingApiToken);
        };
        let fingerprint = request_fingerprint(headers);
        self.write_sessions()?
            .validate(session_id, role, &fingerprint)
            .map_err(session_error)
    }

    fn user_by_email(&self, email: &EmailAddress) -> Result<User, ApiRejection> {
        let data = self.read_data()?;
        data.users
            .get(email.as_str())
            .cloned()
            .ok_or(ApiRejection::NotFound)
    }

    fn provider_by_id(&self, id: &str) -> Result<ProviderAccount, ApiRejection> {
        let data = self.read_data()?;
        data.providers
            .iter()
            .find(|provider| provider_key(provider) == id)
            .cloned()
            .ok_or(ApiRejection::NotFound)
    }

    fn provider_for_inbound_message(
        &self,
        message: &InboundMessage,
    ) -> Result<ProviderAccount, ApiRejection> {
        match message.provider_account_id {
            Some(id) => self.provider_by_id(&serialized_key(&id)),
            None => self.provider_for_mailbox_id(message.mailbox_id),
        }
    }

    fn provider_for_mailbox_id(
        &self,
        mailbox_id: MailboxId,
    ) -> Result<ProviderAccount, ApiRejection> {
        let mailbox = self.mailbox_by_id(mailbox_id)?;
        let data = self.read_data()?;
        data.providers
            .iter()
            .find(|provider| provider.serves_domain(mailbox.address().domain()))
            .cloned()
            .ok_or(ApiRejection::NoProviderForDomain)
    }

    fn replace_provider(
        &self,
        provider: ProviderAccount,
        binding: Option<WebhookBinding>,
    ) -> Result<(), ApiRejection> {
        let mut data = self.write_data()?;
        let key = provider_key(&provider);
        data.providers
            .retain(|candidate| provider_key(candidate) != key);
        data.webhook_bindings.remove(&key);
        insert_webhook_binding(&mut data, &provider, binding);
        data.providers.push(provider);
        Ok(())
    }

    fn remove_provider(&self, id: &str) -> Result<(), ApiRejection> {
        let mut data = self.write_data()?;
        data.providers
            .retain(|provider| provider_key(provider) != id);
        data.webhook_bindings.remove(id);
        Ok(())
    }

    async fn deliver_with_provider(
        &self,
        provider: &ProviderAccount,
        request: SendMailRequest,
    ) -> Result<ProviderSendReceipt, ApiRejection> {
        match provider.provider_type() {
            ProviderType::Resend => self.deliver_with_resend(provider, request).await,
        }
    }

    async fn deliver_with_resend(
        &self,
        provider: &ProviderAccount,
        request: SendMailRequest,
    ) -> Result<ProviderSendReceipt, ApiRejection> {
        let api_key = provider
            .api_key()
            .ok_or(ApiRejection::ProviderApiKeyMissing)?;
        let endpoint = self.resend_endpoint()?;
        ResendProvider::with_endpoint(SecretString::new(api_key.to_string()), endpoint)
            .send(request)
            .await
            .map_err(provider_error)
    }

    fn resend_endpoint(&self) -> Result<String, ApiRejection> {
        self.inner
            .resend_endpoint
            .read()
            .map(|value| value.clone())
            .map_err(|_| ApiRejection::StateUnavailable)
    }

    fn mailbox_by_email(&self, email: &EmailAddress) -> Result<Mailbox, ApiRejection> {
        let data = self.read_data()?;
        data.mailboxes
            .get(email.as_str())
            .cloned()
            .ok_or(ApiRejection::NotFound)
    }

    fn mailbox_by_id(&self, id: MailboxId) -> Result<Mailbox, ApiRejection> {
        let data = self.read_data()?;
        data.mailboxes
            .values()
            .find(|mailbox| mailbox.id() == id)
            .cloned()
            .ok_or(ApiRejection::NotFound)
    }

    fn optional_mailbox_by_email(
        &self,
        email: &EmailAddress,
    ) -> Result<Option<Mailbox>, ApiRejection> {
        let data = self.read_data()?;
        Ok(data.mailboxes.get(email.as_str()).cloned())
    }

    fn ensure_user_can_send(
        &self,
        user_email: &EmailAddress,
        from: &EmailAddress,
    ) -> Result<(), ApiRejection> {
        let user = self.user_by_email(user_email)?;
        ensure_active_user(&user)?;
        let mailbox = self.mailbox_by_email(from)?;
        ensure_owned_mailbox(&mailbox, user.id())?;
        ensure_outbound_mailbox(&mailbox)
    }

    fn insert_mailbox(&self, mailbox: Mailbox) -> Result<(), ApiRejection> {
        let mut data = self.write_data()?;
        data.mailboxes
            .insert(mailbox.address().as_str().to_string(), mailbox);
        Ok(())
    }

    fn owner_id(
        &self,
        owner_email: &EmailAddress,
    ) -> Result<rnovemail_domain::UserId, ApiRejection> {
        let data = self.read_data()?;
        data.users
            .get(owner_email.as_str())
            .map(User::id)
            .ok_or(ApiRejection::NotFound)
    }

    fn read_data(&self) -> Result<std::sync::RwLockReadGuard<'_, AppData>, ApiRejection> {
        self.inner
            .data
            .read()
            .map_err(|_| ApiRejection::StateUnavailable)
    }

    fn write_data(&self) -> Result<std::sync::RwLockWriteGuard<'_, AppData>, ApiRejection> {
        self.inner
            .data
            .write()
            .map_err(|_| ApiRejection::StateUnavailable)
    }

    fn write_sessions(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<'_, SessionRegistry>, ApiRejection> {
        self.inner
            .sessions
            .write()
            .map_err(|_| ApiRejection::StateUnavailable)
    }

    async fn record_audit(&self, action: &str, target: &str) -> Result<(), ApiRejection> {
        let event = AuditEvent::new(
            AuditActor::System,
            action,
            target,
            "admin-api",
            AuditResult::Accepted,
        );
        if let Some(store) = self.store() {
            store
                .append_audit(event.clone())
                .await
                .map_err(store_error)?;
        }
        let mut data = self.write_data()?;
        data.audit_events.push(event);
        Ok(())
    }
}

pub(crate) struct UserPatch {
    pub display_name: Option<String>,
    pub roles: Option<Vec<UserRole>>,
    pub status: Option<UserStatus>,
    pub login_secret_hash: Option<String>,
}

impl UserPatch {
    fn apply(self, user: &mut User) {
        if let Some(display_name) = self.display_name {
            user.set_display_name(display_name);
        }
        if let Some(roles) = self.roles {
            user.set_roles(roles);
        }
        if let Some(status) = self.status {
            user.set_status(status);
        }
        if let Some(hash) = self.login_secret_hash {
            user.set_login_secret_hash(Some(hash));
        }
    }
}

pub(crate) struct ProviderPatch {
    pub name: Option<String>,
    pub domains: Option<Vec<DomainName>>,
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub webhook_secret: Option<String>,
}

impl ProviderPatch {
    fn apply(self, provider: &mut ProviderAccount) {
        if let Some(name) = self.name {
            provider.set_name(name);
        }
        if let Some(domains) = self.domains {
            provider.set_domains(domains);
        }
        if let Some(enabled) = self.enabled {
            provider.set_enabled(enabled);
        }
        if let Some(api_key) = self.api_key {
            provider.replace_api_key(api_key);
        }
        if let Some(webhook_secret) = self.webhook_secret {
            provider.replace_webhook_secret(webhook_secret);
        }
    }
}

pub(crate) struct MailboxPatch {
    pub status: Option<rnovemail_domain::MailboxStatus>,
    pub inbound_enabled: Option<bool>,
    pub outbound_enabled: Option<bool>,
}

impl MailboxPatch {
    fn apply(self, mailbox: &mut Mailbox) {
        if let Some(status) = self.status {
            mailbox.set_status(status);
        }
        if let Some(enabled) = self.inbound_enabled {
            mailbox.set_inbound_enabled(enabled);
        }
        if let Some(enabled) = self.outbound_enabled {
            mailbox.set_outbound_enabled(enabled);
        }
    }
}

fn webhook_binding(
    provider_type: ProviderType,
    webhook_secret: Option<&str>,
) -> Result<Option<WebhookBinding>, ApiRejection> {
    webhook_secret
        .map(|secret| WebhookBinding::new(provider_type, secret))
        .transpose()
}

fn provider_webhook_binding(
    provider: &ProviderAccount,
) -> Result<Option<WebhookBinding>, ApiRejection> {
    webhook_binding(provider.provider_type(), provider.webhook_secret())
}

impl WebhookBinding {
    fn new(provider_type: ProviderType, secret: &str) -> Result<Self, ApiRejection> {
        SignatureVerifier::from_svix_secret(secret).map_err(|_| ApiRejection::BadRequest)?;
        Ok(Self {
            provider_type,
            secret: SecretString::new(secret.to_string()),
        })
    }
}

fn insert_webhook_binding(
    data: &mut AppData,
    provider: &ProviderAccount,
    binding: Option<WebhookBinding>,
) {
    if let Some(binding) = binding {
        data.webhook_bindings
            .insert(provider_key(provider), binding);
    }
}

fn provider_key(provider: &ProviderAccount) -> String {
    serialized_key(&provider.id())
}

fn serialized_key<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| fallback_key(value))
}

fn fallback_key<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "unknown".to_string())
}

fn ensure_provider_matches(
    provider: &str,
    provider_type: ProviderType,
) -> Result<(), ApiRejection> {
    match provider.eq_ignore_ascii_case(provider_type_name(provider_type)) {
        true => Ok(()),
        false => Err(ApiRejection::InvalidWebhookSignature),
    }
}

fn verify_webhook(
    binding: WebhookBinding,
    request: ProviderWebhookRequest,
) -> Result<Vec<ProviderEvent>, ApiRejection> {
    match binding.provider_type {
        ProviderType::Resend => verify_resend_webhook(binding, request),
    }
}

fn verify_resend_webhook(
    binding: WebhookBinding,
    request: ProviderWebhookRequest,
) -> Result<Vec<ProviderEvent>, ApiRejection> {
    let provider = ResendProvider::with_webhook_secret(
        SecretString::new("webhook-only".to_string()),
        binding.secret.expose_secret(),
    )
    .map_err(|_| ApiRejection::InvalidWebhookSignature)?;
    let verified = provider
        .verify_webhook(request)
        .map_err(|_| ApiRejection::InvalidWebhookSignature)?;
    provider
        .map_webhook(verified)
        .map_err(|_| ApiRejection::BadRequest)
}

async fn remember_persistent_event(
    store: Arc<dyn AppStore>,
    provider: &str,
    event_id: &str,
) -> Result<(), ApiRejection> {
    match store
        .remember_event(provider, event_id)
        .await
        .map_err(store_error)?
    {
        true => Ok(()),
        false => Err(ApiRejection::DuplicateWebhookEvent),
    }
}

fn provider_event_id(event: &ProviderEvent) -> &str {
    match event {
        ProviderEvent::Delivered { provider_event_id }
        | ProviderEvent::Bounced { provider_event_id }
        | ProviderEvent::Complained { provider_event_id }
        | ProviderEvent::Inbound {
            provider_event_id, ..
        } => provider_event_id,
    }
}

fn webhook_event_key(provider: &str, event_id: &str) -> String {
    format!("{provider}:{event_id}")
}

fn provider_type_name(provider_type: ProviderType) -> &'static str {
    match provider_type {
        ProviderType::Resend => "resend",
    }
}

async fn load_app_data(store: &dyn AppStore) -> Result<AppData, StoreError> {
    let users = store.list_users().await?;
    let domains = store.list_domains().await?;
    let providers = store.list_providers().await?;
    let mailboxes = store.list_mailboxes().await?;
    let outbound_messages = store.list_outbound().await?;
    let inbound_messages = store.list_inbound().await?;
    let audit_events = store.list_audit().await?;
    let webhook_bindings = webhook_bindings(&providers)?;
    Ok(AppData {
        users: keyed_users(users),
        domains: domains.into_iter().collect(),
        providers,
        mailboxes: keyed_mailboxes(mailboxes),
        outbound_messages: keyed_outbound_messages(outbound_messages),
        inbound_messages: keyed_inbound_messages(inbound_messages),
        webhook_bindings,
        webhook_events: HashSet::new(),
        audit_events,
    })
}

fn webhook_bindings(
    providers: &[ProviderAccount],
) -> Result<HashMap<String, WebhookBinding>, StoreError> {
    let mut bindings = HashMap::new();
    for provider in providers {
        append_webhook_binding(provider, &mut bindings)?;
    }
    Ok(bindings)
}

fn append_webhook_binding(
    provider: &ProviderAccount,
    bindings: &mut HashMap<String, WebhookBinding>,
) -> Result<(), StoreError> {
    let binding = provider_webhook_binding(provider).map_err(|_| StoreError::OperationFailed)?;
    if let Some(binding) = binding {
        bindings.insert(provider_key(provider), binding);
    }
    Ok(())
}

fn keyed_users(users: Vec<User>) -> HashMap<String, User> {
    users
        .into_iter()
        .map(|user| (user.primary_email().as_str().to_string(), user))
        .collect()
}

fn keyed_mailboxes(mailboxes: Vec<Mailbox>) -> HashMap<String, Mailbox> {
    mailboxes
        .into_iter()
        .map(|mailbox| (mailbox.address().as_str().to_string(), mailbox))
        .collect()
}

fn keyed_outbound_messages(messages: Vec<OutboundMessage>) -> HashMap<String, OutboundMessage> {
    messages
        .into_iter()
        .map(|message| (serialized_key(&message.id), message))
        .collect()
}

fn keyed_inbound_messages(messages: Vec<InboundMessage>) -> HashMap<String, InboundMessage> {
    messages
        .into_iter()
        .map(|message| (serialized_key(&message.id), message))
        .collect()
}

fn store_error(_error: StoreError) -> ApiRejection {
    ApiRejection::StateUnavailable
}

fn provider_error(error: ProviderError) -> ApiRejection {
    match error {
        ProviderError::NoProviderForDomain => ApiRejection::NoProviderForDomain,
        ProviderError::MissingField(_) | ProviderError::EmptyRecipients => ApiRejection::BadRequest,
        ProviderError::InvalidPayload => ApiRejection::BadRequest,
        ProviderError::InvalidSignature => ApiRejection::InvalidWebhookSignature,
        ProviderError::ProviderRejected => ApiRejection::ProviderRejected,
    }
}

fn default_resend_endpoint() -> String {
    "https://api.resend.com".to_string()
}

fn normalize_public_base_url(value: String) -> Option<String> {
    let trimmed = value.trim().trim_end_matches('/');
    match trimmed.is_empty() {
        true => None,
        false => Some(trimmed.to_string()),
    }
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(root_redirect))
        .route("/api/openapi.json", get(openapi_json))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .merge(admin_pages::routes())
        .merge(session_routes::routes())
        .merge(user_pages::routes())
        .merge(admin_routes::routes())
        .merge(mail_routes::routes())
        .merge(webhook_routes::routes())
        .with_state(state)
}

async fn root_redirect() -> Redirect {
    Redirect::to("/portal")
}

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(openapi::openapi())
}

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz() -> &'static str {
    "ready"
}

fn optional_bearer_token(headers: &HeaderMap) -> Result<Option<&str>, ApiRejection> {
    let Some(value) = headers.get("authorization") else {
        return Ok(None);
    };
    let text = value.to_str().map_err(|_| ApiRejection::InvalidApiToken)?;
    text.strip_prefix("Bearer ")
        .map(Some)
        .ok_or(ApiRejection::InvalidApiToken)
}

fn session_cookie(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(find_session_cookie)
}

fn find_session_cookie(cookies: &str) -> Option<&str> {
    cookies.split(';').find_map(cookie_value)
}

fn cookie_value(cookie: &str) -> Option<&str> {
    let (name, value) = cookie.trim().split_once('=')?;
    match name == SESSION_COOKIE {
        true => Some(value),
        false => None,
    }
}

fn request_fingerprint(headers: &HeaderMap) -> String {
    let user_agent = header_text(headers, "user-agent");
    let forwarded_for = header_text(headers, "x-forwarded-for");
    format!("{user_agent}|{forwarded_for}")
}

fn header_text<'a>(headers: &'a HeaderMap, name: &str) -> &'a str {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
}

fn session_error(error: SessionError) -> ApiRejection {
    match error {
        SessionError::Locked => ApiRejection::TooManyLoginAttempts,
        SessionError::Missing => ApiRejection::MissingApiToken,
        SessionError::Invalid => ApiRejection::InvalidApiToken,
    }
}

fn ensure_active_user(user: &User) -> Result<(), ApiRejection> {
    match user.status() {
        UserStatus::Active => Ok(()),
        UserStatus::Disabled => Err(ApiRejection::InvalidApiToken),
    }
}

fn ensure_owned_mailbox(
    mailbox: &Mailbox,
    owner_id: rnovemail_domain::UserId,
) -> Result<(), ApiRejection> {
    match mailbox.owner_id() == owner_id {
        true => Ok(()),
        false => Err(ApiRejection::MailboxAccessDenied),
    }
}

fn ensure_outbound_mailbox(mailbox: &Mailbox) -> Result<(), ApiRejection> {
    match mailbox.status() == MailboxStatus::Active && mailbox.outbound_enabled() {
        true => Ok(()),
        false => Err(ApiRejection::MailboxAccessDenied),
    }
}

fn is_inbound_mailbox(mailbox: &Mailbox) -> bool {
    mailbox.status() == MailboxStatus::Active && mailbox.inbound_enabled()
}

fn outbound_message(
    provider: &ProviderAccount,
    request: &SendMailRequest,
    receipt: &ProviderSendReceipt,
) -> OutboundMessage {
    OutboundMessage {
        id: receipt.message_id,
        provider_account_id: provider.id(),
        from: request.from().clone(),
        to: request.to().to_vec(),
        subject: request.subject().to_string(),
        text: request.text().to_string(),
        status: MessageStatus::Sent,
        timeline: vec![timeline_entry(
            MessageStatus::Sent,
            &receipt.provider_message_id,
        )],
    }
}

fn inbound_message(
    provider_account_id: rnovemail_domain::ProviderAccountId,
    provider_event_id: &str,
    from: &EmailAddress,
    mailbox: &Mailbox,
    subject: &str,
    text: &str,
    detail: Option<InboundMessageDetail>,
) -> InboundMessage {
    let display = inbound_display(provider_event_id, subject, text, detail);
    InboundMessage {
        id: rnovemail_domain::MessageId::new(),
        mailbox_id: mailbox.id(),
        provider_account_id: Some(provider_account_id),
        provider_event_id: display.provider_event_id,
        from: from.clone(),
        subject: display.subject,
        text: display.text,
        received_at: Utc::now(),
        detail: display.detail,
    }
}

struct InboundDisplay {
    provider_event_id: String,
    subject: String,
    text: String,
    detail: Option<InboundMessageDetail>,
}

fn inbound_display(
    provider_event_id: &str,
    subject: &str,
    text: &str,
    detail: Option<InboundMessageDetail>,
) -> InboundDisplay {
    InboundDisplay {
        provider_event_id: provider_event_id.to_string(),
        subject: inbound_subject_text(subject, detail.as_ref()),
        text: inbound_body_text(text, detail.as_ref()),
        detail,
    }
}

fn inbound_subject_text(subject: &str, detail: Option<&InboundMessageDetail>) -> String {
    detail
        .map(|detail| detail.subject.trim())
        .filter(|subject| !subject.is_empty())
        .unwrap_or(subject)
        .to_string()
}

fn inbound_body_text(text: &str, detail: Option<&InboundMessageDetail>) -> String {
    detail
        .map(|detail| detail.text.trim())
        .filter(|text| !text.is_empty())
        .unwrap_or(text)
        .to_string()
}

fn timeline_entry(status: MessageStatus, note: &str) -> MessageTimelineEntry {
    MessageTimelineEntry {
        status,
        at: Utc::now(),
        note: note.to_string(),
    }
}
