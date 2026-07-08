use std::collections::HashMap;

use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use rnovemail_admin::{
    AdminData, AdminSection, AuditRow, DomainRow, Lang, MailboxRow, PageContext, ProviderRow,
    Theme, UserRow,
};
use rnovemail_domain::{Mailbox, ProviderAccount, ProviderType, User, UserId};
use serde::Deserialize;

use crate::{AppState, middleware::ApiRejection};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin", get(dashboard))
        .route("/admin/users", get(users))
        .route("/admin/domains", get(domains))
        .route("/admin/providers", get(providers))
        .route("/admin/mailboxes", get(mailboxes))
        .route("/admin/audit", get(audit))
}

#[derive(Deserialize)]
struct PageQuery {
    lang: Option<String>,
    theme: Option<String>,
}

async fn dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Response {
    admin_response(state, headers, query, AdminSection::Dashboard)
}

async fn users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Response {
    admin_response(state, headers, query, AdminSection::Users)
}

async fn domains(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Response {
    admin_response(state, headers, query, AdminSection::Domains)
}

async fn providers(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Response {
    admin_response(state, headers, query, AdminSection::Providers)
}

async fn mailboxes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Response {
    admin_response(state, headers, query, AdminSection::Mailboxes)
}

async fn audit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Response {
    admin_response(state, headers, query, AdminSection::Audit)
}

fn admin_response(
    state: AppState,
    headers: HeaderMap,
    query: PageQuery,
    section: AdminSection,
) -> Response {
    if state.admin_principal(&headers).is_err() {
        return Redirect::to(&login_location(section)).into_response();
    }
    let ctx = page_context(&query, section_path(section));
    match admin_data(&state) {
        Ok(data) => rnovemail_admin::admin_page(&ctx, section, &data).into_response(),
        Err(error) => error.into_response(),
    }
}

fn admin_data(state: &AppState) -> Result<AdminData, ApiRejection> {
    let users = state.list_users()?;
    let owners = owner_lookup(&users);
    Ok(AdminData {
        users: user_rows(users),
        domains: domain_rows(state)?,
        providers: provider_rows(state)?,
        mailboxes: mailbox_rows(state.list_mailboxes()?, &owners),
        audit_events: audit_rows(state)?,
    })
}

fn user_rows(users: Vec<User>) -> Vec<UserRow> {
    sorted(
        users
            .into_iter()
            .map(|user| UserRow {
                display_name: user.display_name().to_string(),
                email: user.primary_email().as_str().to_string(),
                roles: roles_text(user.roles()),
                status: format!("{:?}", user.status()),
            })
            .collect(),
        |row| row.email.clone(),
    )
}

fn domain_rows(state: &AppState) -> Result<Vec<DomainRow>, ApiRejection> {
    Ok(sorted(
        state
            .list_domains()?
            .into_iter()
            .map(|domain| DomainRow {
                domain: domain.as_str().to_string(),
            })
            .collect(),
        |row| row.domain.clone(),
    ))
}

fn provider_rows(state: &AppState) -> Result<Vec<ProviderRow>, ApiRejection> {
    Ok(sorted(
        state
            .list_providers()?
            .into_iter()
            .map(provider_row)
            .collect(),
        |row| row.name.clone(),
    ))
}

fn provider_row(provider: ProviderAccount) -> ProviderRow {
    ProviderRow {
        id: serialized_key(&provider.id()),
        name: provider.name().to_string(),
        provider_type: provider_type(provider.provider_type()).to_string(),
        domains: provider
            .domains()
            .iter()
            .map(|domain| domain.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        enabled: provider.enabled(),
    }
}

fn mailbox_rows(mailboxes: Vec<Mailbox>, owners: &HashMap<UserId, String>) -> Vec<MailboxRow> {
    sorted(
        mailboxes
            .into_iter()
            .map(|mailbox| MailboxRow {
                owner: owners
                    .get(&mailbox.owner_id())
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string()),
                email: mailbox.address().as_str().to_string(),
                status: format!("{:?}", mailbox.status()),
                inbound_enabled: mailbox.inbound_enabled(),
                outbound_enabled: mailbox.outbound_enabled(),
            })
            .collect(),
        |row| row.email.clone(),
    )
}

fn audit_rows(state: &AppState) -> Result<Vec<AuditRow>, ApiRejection> {
    Ok(state
        .list_audit()?
        .into_iter()
        .map(|event| AuditRow {
            at: event.at.to_rfc3339(),
            action: event.action,
            target: event.target,
            result: format!("{:?}", event.result),
        })
        .collect())
}

fn owner_lookup(users: &[User]) -> HashMap<UserId, String> {
    users
        .iter()
        .map(|user| (user.id(), user.primary_email().as_str().to_string()))
        .collect()
}

fn page_context(query: &PageQuery, next: &str) -> PageContext {
    PageContext {
        lang: Lang::parse(query.lang.as_deref()),
        theme: Theme::parse(query.theme.as_deref()),
        next: next.to_string(),
    }
}

fn login_location(section: AdminSection) -> String {
    format!(
        "/login?scope=admin&next={}",
        encoded_next(section_path(section))
    )
}

fn section_path(section: AdminSection) -> &'static str {
    match section {
        AdminSection::Audit => "/admin/audit",
        AdminSection::Dashboard => "/admin",
        AdminSection::Domains => "/admin/domains",
        AdminSection::Mailboxes => "/admin/mailboxes",
        AdminSection::Providers => "/admin/providers",
        AdminSection::Users => "/admin/users",
    }
}

fn encoded_next(path: &str) -> String {
    path.replace('/', "%2F")
}

fn roles_text(roles: &[rnovemail_domain::UserRole]) -> String {
    roles
        .iter()
        .map(|role| format!("{role:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn provider_type(provider_type: ProviderType) -> &'static str {
    match provider_type {
        ProviderType::Resend => "resend",
    }
}

fn serialized_key<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| serde_json::to_string(value).unwrap_or_else(|_| "unknown".to_string()))
}

fn sorted<T, F>(mut values: Vec<T>, key: F) -> Vec<T>
where
    F: Fn(&T) -> String,
{
    values.sort_by_key(key);
    values
}
