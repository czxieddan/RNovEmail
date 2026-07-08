use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{patch, post},
};
use rnovemail_auth::hash_login_secret;
use rnovemail_domain::{
    DomainName, EmailAddress, MailboxStatus, ProviderAccount, ProviderType, UserRole, UserStatus,
};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};

use crate::router::{MailboxPatch, ProviderPatch, UserPatch};
use crate::{AppState, middleware::ApiRejection};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/admin/users", post(create_user))
        .route("/api/v1/admin/users/{id}", patch(update_user))
        .route("/api/v1/admin/domains", post(create_domain))
        .route(
            "/api/v1/admin/domains/{id}",
            patch(update_domain).delete(delete_domain),
        )
        .route("/api/v1/admin/provider-accounts", post(create_provider))
        .route(
            "/api/v1/admin/provider-accounts/{id}",
            patch(update_provider).delete(delete_provider),
        )
        .route("/api/v1/admin/mailboxes", post(create_mailbox))
        .route("/api/v1/admin/mailboxes/{id}", patch(update_mailbox))
}

async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateUserRequest>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    create_user_response(&state, request).await.into_response()
}

async fn update_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<UpdateUserRequest>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    update_user_response(&state, id, request)
        .await
        .into_response()
}

async fn create_domain(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateDomainRequest>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    create_domain_response(&state, request)
        .await
        .into_response()
}

async fn update_domain(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<UpdateDomainRequest>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    update_domain_response(&state, id, request)
        .await
        .into_response()
}

async fn delete_domain(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    delete_domain_response(&state, id).await.into_response()
}

async fn create_provider(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateProviderRequest>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    create_provider_response(&state, request)
        .await
        .into_response()
}

async fn update_provider(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<UpdateProviderRequest>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    update_provider_response(&state, id, request)
        .await
        .into_response()
}

async fn delete_provider(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    delete_provider_response(&state, id).await.into_response()
}

async fn create_mailbox(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateMailboxRequest>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    create_mailbox_response(&state, request)
        .await
        .into_response()
}

async fn update_mailbox(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<UpdateMailboxRequest>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    update_mailbox_response(&state, id, request)
        .await
        .into_response()
}

async fn create_user_response(
    state: &AppState,
    request: CreateUserRequest,
) -> Result<Json<UserResponse>, ApiRejection> {
    let email = EmailAddress::parse(&request.email).map_err(|_| ApiRejection::BadRequest)?;
    let login_secret_hash = login_secret_hash(request.login_secret)?;
    let user = state
        .add_user_with_secret(
            request.display_name,
            email,
            request.roles,
            login_secret_hash,
        )
        .await?;
    Ok(Json(UserResponse::from_user(&user)))
}

async fn update_user_response(
    state: &AppState,
    id: String,
    request: UpdateUserRequest,
) -> Result<Json<UserResponse>, ApiRejection> {
    let email = EmailAddress::parse(id).map_err(|_| ApiRejection::BadRequest)?;
    let patch = UserPatch {
        display_name: request.display_name,
        roles: request.roles,
        status: request.status,
        login_secret_hash: login_secret_hash(request.login_secret)?,
    };
    let user = state.update_user(&email, patch).await?;
    Ok(Json(UserResponse::from_user(&user)))
}

async fn create_domain_response(
    state: &AppState,
    request: CreateDomainRequest,
) -> Result<Json<DomainResponse>, ApiRejection> {
    let domain = DomainName::parse(&request.domain).map_err(|_| ApiRejection::BadRequest)?;
    let domain = state.add_domain(domain).await?;
    Ok(Json(DomainResponse {
        domain: domain.as_str().to_string(),
    }))
}

async fn create_provider_response(
    state: &AppState,
    request: CreateProviderRequest,
) -> Result<Json<ProviderResponse>, ApiRejection> {
    let provider_type = parse_provider(&request.provider_type)?;
    let domains = parse_domains(request.domains)?;
    let provider = ProviderAccount::new(provider_type, request.name, domains)
        .with_api_key(optional_secret(request.api_key))
        .with_webhook_secret(optional_secret(request.webhook_secret));
    let provider = state.add_provider(provider).await?;
    Ok(Json(ProviderResponse::from_provider(&provider)))
}

async fn update_provider_response(
    state: &AppState,
    id: String,
    request: UpdateProviderRequest,
) -> Result<Json<ProviderResponse>, ApiRejection> {
    let domains = optional_domains(request.domains)?;
    let patch = ProviderPatch {
        name: request.name,
        domains,
        enabled: request.enabled,
        api_key: optional_secret(request.api_key),
        webhook_secret: optional_secret(request.webhook_secret),
    };
    let provider = state.update_provider(&id, patch).await?;
    Ok(Json(ProviderResponse::from_provider(&provider)))
}

async fn update_domain_response(
    state: &AppState,
    id: String,
    request: UpdateDomainRequest,
) -> Result<Json<DomainResponse>, ApiRejection> {
    let old_domain = DomainName::parse(id).map_err(|_| ApiRejection::BadRequest)?;
    let new_domain = DomainName::parse(request.domain).map_err(|_| ApiRejection::BadRequest)?;
    let domain = state.update_domain(old_domain, new_domain).await?;
    Ok(Json(DomainResponse {
        domain: domain.as_str().to_string(),
    }))
}

async fn delete_domain_response(state: &AppState, id: String) -> Result<StatusCode, ApiRejection> {
    let domain = DomainName::parse(id).map_err(|_| ApiRejection::BadRequest)?;
    state.delete_domain(domain).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_provider_response(
    state: &AppState,
    id: String,
) -> Result<StatusCode, ApiRejection> {
    state.delete_provider(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create_mailbox_response(
    state: &AppState,
    request: CreateMailboxRequest,
) -> Result<Json<MailboxResponse>, ApiRejection> {
    let owner = EmailAddress::parse(&request.owner_email).map_err(|_| ApiRejection::BadRequest)?;
    let mailbox =
        EmailAddress::parse(&request.mailbox_email).map_err(|_| ApiRejection::BadRequest)?;
    let mailbox = state.add_mailbox(owner, mailbox).await?;
    Ok(Json(MailboxResponse {
        email: mailbox.address().as_str().to_string(),
    }))
}

async fn update_mailbox_response(
    state: &AppState,
    id: String,
    request: UpdateMailboxRequest,
) -> Result<Json<MailboxResponse>, ApiRejection> {
    let email = EmailAddress::parse(id).map_err(|_| ApiRejection::BadRequest)?;
    let patch = MailboxPatch {
        status: request.status,
        inbound_enabled: request.inbound_enabled,
        outbound_enabled: request.outbound_enabled,
    };
    let mailbox = state.update_mailbox(&email, patch).await?;
    Ok(Json(MailboxResponse {
        email: mailbox.address().as_str().to_string(),
    }))
}

fn parse_provider(value: &str) -> Result<ProviderType, ApiRejection> {
    match value.eq_ignore_ascii_case("resend") {
        true => Ok(ProviderType::Resend),
        false => Err(ApiRejection::BadRequest),
    }
}

fn parse_domains(values: Vec<String>) -> Result<Vec<DomainName>, ApiRejection> {
    values
        .into_iter()
        .map(|value| DomainName::parse(value).map_err(|_| ApiRejection::BadRequest))
        .collect()
}

fn optional_domains(values: Option<Vec<String>>) -> Result<Option<Vec<DomainName>>, ApiRejection> {
    values.map(parse_domains).transpose()
}

fn login_secret_hash(secret: Option<String>) -> Result<Option<String>, ApiRejection> {
    secret
        .filter(|value| !value.trim().is_empty())
        .map(|value| hash_login_secret(&SecretString::new(value)))
        .transpose()
        .map_err(|_| ApiRejection::BadRequest)
}

fn optional_secret(secret: Option<String>) -> Option<String> {
    secret
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[derive(Deserialize)]
struct CreateUserRequest {
    display_name: String,
    email: String,
    #[serde(default)]
    roles: Vec<UserRole>,
    login_secret: Option<String>,
}

#[derive(Deserialize)]
struct UpdateUserRequest {
    display_name: Option<String>,
    roles: Option<Vec<UserRole>>,
    status: Option<UserStatus>,
    login_secret: Option<String>,
}

#[derive(Deserialize)]
struct CreateDomainRequest {
    domain: String,
}

#[derive(Deserialize)]
struct UpdateDomainRequest {
    domain: String,
}

#[derive(Deserialize)]
struct CreateProviderRequest {
    name: String,
    provider_type: String,
    domains: Vec<String>,
    api_key: Option<String>,
    webhook_secret: Option<String>,
}

#[derive(Deserialize)]
struct UpdateProviderRequest {
    name: Option<String>,
    domains: Option<Vec<String>>,
    enabled: Option<bool>,
    api_key: Option<String>,
    webhook_secret: Option<String>,
}

#[derive(Deserialize)]
struct CreateMailboxRequest {
    owner_email: String,
    mailbox_email: String,
}

#[derive(Deserialize)]
struct UpdateMailboxRequest {
    status: Option<MailboxStatus>,
    inbound_enabled: Option<bool>,
    outbound_enabled: Option<bool>,
}

#[derive(Serialize)]
struct UserResponse {
    email: String,
    status: String,
}

impl UserResponse {
    fn from_user(user: &rnovemail_domain::User) -> Self {
        Self {
            email: user.primary_email().as_str().to_string(),
            status: "active".to_string(),
        }
    }
}

#[derive(Serialize)]
struct DomainResponse {
    domain: String,
}

#[derive(Serialize)]
struct ProviderResponse {
    id: String,
    provider_type: String,
    api_key_configured: bool,
}

impl ProviderResponse {
    fn from_provider(provider: &ProviderAccount) -> Self {
        match provider.provider_type() {
            ProviderType::Resend => Self {
                id: serialized_key(&provider.id()),
                provider_type: "resend".to_string(),
                api_key_configured: provider.api_key_configured(),
            },
        }
    }
}

#[derive(Serialize)]
struct MailboxResponse {
    email: String,
}

fn serialized_key<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| fallback_key(value))
}

fn fallback_key<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "unknown".to_string())
}
