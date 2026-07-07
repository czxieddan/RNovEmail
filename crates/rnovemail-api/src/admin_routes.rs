use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::{patch, post},
};
use rnovemail_domain::{DomainName, EmailAddress, ProviderAccount, ProviderType, UserRole};
use serde::{Deserialize, Serialize};

use crate::{AppState, middleware::ApiRejection};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/admin/users", post(create_user))
        .route("/api/v1/admin/users/{id}", patch(update_user))
        .route("/api/v1/admin/domains", post(create_domain))
        .route("/api/v1/admin/domains/{id}", patch(update_domain))
        .route("/api/v1/admin/provider-accounts", post(create_provider))
        .route(
            "/api/v1/admin/provider-accounts/{id}",
            patch(update_provider),
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
    create_user_response(&state, request).into_response()
}

async fn update_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(_id): Path<String>,
) -> Response {
    admin_accept(&state, headers, "user_updated")
}

async fn create_domain(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateDomainRequest>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    create_domain_response(&state, request).into_response()
}

async fn update_domain(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(_id): Path<String>,
) -> Response {
    admin_accept(&state, headers, "domain_updated")
}

async fn create_provider(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateProviderRequest>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    create_provider_response(&state, request).into_response()
}

async fn update_provider(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(_id): Path<String>,
) -> Response {
    admin_accept(&state, headers, "provider_updated")
}

async fn create_mailbox(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateMailboxRequest>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    create_mailbox_response(&state, request).into_response()
}

async fn update_mailbox(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(_id): Path<String>,
) -> Response {
    admin_accept(&state, headers, "mailbox_updated")
}

fn admin_accept(state: &AppState, headers: HeaderMap, accepted: &'static str) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    axum::Json(serde_json::json!({ "status": accepted })).into_response()
}

fn create_user_response(
    state: &AppState,
    request: CreateUserRequest,
) -> Result<Json<UserResponse>, ApiRejection> {
    let email = EmailAddress::parse(&request.email).map_err(|_| ApiRejection::BadRequest)?;
    let user = state.add_user(request.display_name, email, request.roles)?;
    Ok(Json(UserResponse::from_user(&user)))
}

fn create_domain_response(
    state: &AppState,
    request: CreateDomainRequest,
) -> Result<Json<DomainResponse>, ApiRejection> {
    let domain = DomainName::parse(&request.domain).map_err(|_| ApiRejection::BadRequest)?;
    let domain = state.add_domain(domain)?;
    Ok(Json(DomainResponse {
        domain: domain.as_str().to_string(),
    }))
}

fn create_provider_response(
    state: &AppState,
    request: CreateProviderRequest,
) -> Result<Json<ProviderResponse>, ApiRejection> {
    let provider_type = parse_provider(&request.provider_type)?;
    let domains = parse_domains(request.domains)?;
    let provider = ProviderAccount::new(provider_type, request.name, domains);
    let provider = state.add_provider(provider)?;
    Ok(Json(ProviderResponse::from_provider(&provider)))
}

fn create_mailbox_response(
    state: &AppState,
    request: CreateMailboxRequest,
) -> Result<Json<MailboxResponse>, ApiRejection> {
    let owner = EmailAddress::parse(&request.owner_email).map_err(|_| ApiRejection::BadRequest)?;
    let mailbox =
        EmailAddress::parse(&request.mailbox_email).map_err(|_| ApiRejection::BadRequest)?;
    let mailbox = state.add_mailbox(owner, mailbox)?;
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

#[derive(Deserialize)]
struct CreateUserRequest {
    display_name: String,
    email: String,
    #[serde(default)]
    roles: Vec<UserRole>,
}

#[derive(Deserialize)]
struct CreateDomainRequest {
    domain: String,
}

#[derive(Deserialize)]
struct CreateProviderRequest {
    name: String,
    provider_type: String,
    domains: Vec<String>,
}

#[derive(Deserialize)]
struct CreateMailboxRequest {
    owner_email: String,
    mailbox_email: String,
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
    provider_type: String,
}

impl ProviderResponse {
    fn from_provider(provider: &ProviderAccount) -> Self {
        match provider.provider_type() {
            ProviderType::Resend => Self {
                provider_type: "resend".to_string(),
            },
        }
    }
}

#[derive(Serialize)]
struct MailboxResponse {
    email: String,
}
