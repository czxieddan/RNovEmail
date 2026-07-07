use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use axum::{Json, Router, http::HeaderMap, response::IntoResponse, routing::get};
use rnovemail_domain::{
    DomainName, EmailAddress, Mailbox, ProviderAccount, ProviderType, User, UserRole,
};
use rnovemail_providers::{MailProvider, ProviderEvent, ProviderWebhookRequest, ResendProvider};
use rnovemail_store::{AppStore, StoreError};
use rnovemail_webhook::SignatureVerifier;
use secrecy::{ExposeSecret, SecretString};
use subtle::ConstantTimeEq;

use crate::{admin_routes, mail_routes, middleware::ApiRejection, openapi, webhook_routes};

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    ready: bool,
    admin_token: Option<String>,
    store: Option<Arc<dyn AppStore>>,
    data: RwLock<AppData>,
}

#[derive(Default)]
struct AppData {
    users: HashMap<String, User>,
    domains: HashSet<DomainName>,
    providers: Vec<ProviderAccount>,
    mailboxes: HashMap<String, Mailbox>,
    webhook_bindings: HashMap<String, WebhookBinding>,
    webhook_events: HashSet<String>,
}

#[derive(Clone)]
struct WebhookBinding {
    provider_type: ProviderType,
    secret: SecretString,
}

impl AppState {
    pub fn empty() -> Self {
        Self::new(None)
    }

    pub fn with_admin_token(token: impl Into<String>) -> Self {
        Self::new(Some(token.into()))
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
        let presented = bearer_token(headers)?;
        self.verify_admin_token(presented)
    }

    pub(crate) async fn add_user(
        &self,
        display_name: String,
        email: EmailAddress,
        roles: Vec<UserRole>,
    ) -> Result<User, ApiRejection> {
        let user = User::assign(display_name, email.clone(), roles);
        if let Some(store) = self.store() {
            store.put_user(user.clone()).await.map_err(store_error)?;
        }
        let mut data = self.write_data()?;
        data.users.insert(email.as_str().to_string(), user.clone());
        Ok(user)
    }

    pub(crate) async fn add_domain(&self, domain: DomainName) -> Result<DomainName, ApiRejection> {
        if let Some(store) = self.store() {
            store
                .put_domain(domain.clone())
                .await
                .map_err(store_error)?;
        }
        let mut data = self.write_data()?;
        data.domains.insert(domain.clone());
        Ok(domain)
    }

    pub(crate) async fn add_provider(
        &self,
        provider: ProviderAccount,
        webhook_secret: Option<String>,
    ) -> Result<ProviderAccount, ApiRejection> {
        let binding = webhook_binding(provider.provider_type(), webhook_secret)?;
        if let Some(store) = self.store() {
            store
                .put_provider(provider.clone())
                .await
                .map_err(store_error)?;
        }
        let mut data = self.write_data()?;
        insert_webhook_binding(&mut data, &provider, binding);
        data.providers.push(provider.clone());
        Ok(provider)
    }

    pub(crate) fn provider_for_sender(
        &self,
        sender: &EmailAddress,
    ) -> Result<ProviderAccount, ApiRejection> {
        let data = self.read_data()?;
        data.providers
            .iter()
            .find(|provider| provider.serves_domain(sender.domain()))
            .cloned()
            .ok_or(ApiRejection::NoProviderForDomain)
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
        let mut data = self.write_data()?;
        data.mailboxes
            .insert(mailbox_email.as_str().to_string(), mailbox.clone());
        Ok(mailbox)
    }

    pub(crate) async fn ingest_webhook(
        &self,
        provider: &str,
        account_id: &str,
        request: ProviderWebhookRequest,
    ) -> Result<(), ApiRejection> {
        let binding = self.webhook_binding(account_id)?;
        ensure_provider_matches(provider, binding.provider_type)?;
        let events = verify_webhook(binding, request)?;
        self.remember_webhook_events(provider, events).await
    }

    fn new(admin_token: Option<String>) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                ready: true,
                admin_token,
                store: None,
                data: RwLock::new(AppData::default()),
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
        events: Vec<ProviderEvent>,
    ) -> Result<(), ApiRejection> {
        for event in events {
            self.remember_webhook_event(provider, provider_event_id(&event))
                .await?;
        }
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

    fn verify_admin_token(&self, presented: &str) -> Result<(), ApiRejection> {
        let Some(expected) = &self.inner.admin_token else {
            return Err(ApiRejection::InvalidApiToken);
        };
        match bool::from(expected.as_bytes().ct_eq(presented.as_bytes())) {
            true => Ok(()),
            false => Err(ApiRejection::InvalidApiToken),
        }
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
}

fn webhook_binding(
    provider_type: ProviderType,
    webhook_secret: Option<String>,
) -> Result<Option<WebhookBinding>, ApiRejection> {
    webhook_secret
        .map(|secret| WebhookBinding::new(provider_type, secret))
        .transpose()
}

impl WebhookBinding {
    fn new(provider_type: ProviderType, secret: String) -> Result<Self, ApiRejection> {
        SignatureVerifier::from_svix_secret(&secret).map_err(|_| ApiRejection::BadRequest)?;
        Ok(Self {
            provider_type,
            secret: SecretString::new(secret),
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
        | ProviderEvent::Inbound { provider_event_id } => provider_event_id,
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
    Ok(AppData {
        users: keyed_users(users),
        domains: domains.into_iter().collect(),
        providers,
        mailboxes: keyed_mailboxes(mailboxes),
        webhook_bindings: HashMap::new(),
        webhook_events: HashSet::new(),
    })
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

fn store_error(_error: StoreError) -> ApiRejection {
    ApiRejection::StateUnavailable
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/admin", get(admin_dashboard))
        .route("/admin/login", get(admin_login))
        .route("/api/openapi.json", get(openapi_json))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .merge(admin_routes::routes())
        .merge(mail_routes::routes())
        .merge(webhook_routes::routes())
        .with_state(state)
}

async fn admin_dashboard() -> impl IntoResponse {
    rnovemail_admin::dashboard_page()
}

async fn admin_login() -> impl IntoResponse {
    rnovemail_admin::login_page()
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

fn bearer_token(headers: &HeaderMap) -> Result<&str, ApiRejection> {
    let value = headers
        .get("authorization")
        .ok_or(ApiRejection::MissingApiToken)?;
    let text = value.to_str().map_err(|_| ApiRejection::InvalidApiToken)?;
    text.strip_prefix("Bearer ")
        .ok_or(ApiRejection::InvalidApiToken)
}
