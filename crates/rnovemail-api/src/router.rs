use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use axum::{Router, http::HeaderMap, routing::get};
use rnovemail_domain::{DomainName, EmailAddress, Mailbox, ProviderAccount, User, UserRole};

use crate::{admin_routes, mail_routes, middleware::ApiRejection, webhook_routes};

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    ready: bool,
    admin_token: Option<String>,
    data: RwLock<AppData>,
}

#[derive(Default)]
struct AppData {
    users: HashMap<String, User>,
    domains: HashSet<DomainName>,
    providers: Vec<ProviderAccount>,
    mailboxes: HashMap<String, Mailbox>,
}

impl AppState {
    pub fn empty() -> Self {
        Self::new(None)
    }

    pub fn with_admin_token(token: impl Into<String>) -> Self {
        Self::new(Some(token.into()))
    }

    pub fn ready(&self) -> bool {
        self.inner.ready
    }

    pub(crate) fn require_admin(&self, headers: &HeaderMap) -> Result<(), ApiRejection> {
        let presented = bearer_token(headers)?;
        self.verify_admin_token(presented)
    }

    pub(crate) fn add_user(
        &self,
        display_name: String,
        email: EmailAddress,
        roles: Vec<UserRole>,
    ) -> Result<User, ApiRejection> {
        let user = User::assign(display_name, email.clone(), roles);
        let mut data = self.write_data()?;
        data.users.insert(email.as_str().to_string(), user.clone());
        Ok(user)
    }

    pub(crate) fn add_domain(&self, domain: DomainName) -> Result<DomainName, ApiRejection> {
        let mut data = self.write_data()?;
        data.domains.insert(domain.clone());
        Ok(domain)
    }

    pub(crate) fn add_provider(
        &self,
        provider: ProviderAccount,
    ) -> Result<ProviderAccount, ApiRejection> {
        let mut data = self.write_data()?;
        data.providers.push(provider.clone());
        Ok(provider)
    }

    pub(crate) fn add_mailbox(
        &self,
        owner_email: EmailAddress,
        mailbox_email: EmailAddress,
    ) -> Result<Mailbox, ApiRejection> {
        let owner_id = self.owner_id(&owner_email)?;
        let mailbox = Mailbox::assign(owner_id, mailbox_email.clone());
        let mut data = self.write_data()?;
        data.mailboxes
            .insert(mailbox_email.as_str().to_string(), mailbox.clone());
        Ok(mailbox)
    }

    fn new(admin_token: Option<String>) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                ready: true,
                admin_token,
                data: RwLock::new(AppData::default()),
            }),
        }
    }

    fn verify_admin_token(&self, presented: &str) -> Result<(), ApiRejection> {
        let Some(expected) = &self.inner.admin_token else {
            return Err(ApiRejection::InvalidApiToken);
        };
        match expected == presented {
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

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .merge(admin_routes::routes())
        .merge(mail_routes::routes())
        .merge(webhook_routes::routes())
        .with_state(state)
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
