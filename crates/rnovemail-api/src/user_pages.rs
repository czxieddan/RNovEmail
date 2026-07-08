use std::collections::HashMap;

use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use rnovemail_admin::{Lang, MailboxRow, PageContext, PortalData, Theme};
use rnovemail_domain::{EmailAddress, Mailbox, User, UserId};
use serde::Deserialize;

use crate::{AppState, middleware::ApiRejection};

pub fn routes() -> Router<AppState> {
    Router::new().route("/portal", get(portal))
}

#[derive(Deserialize)]
struct PageQuery {
    lang: Option<String>,
    theme: Option<String>,
}

async fn portal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PageQuery>,
) -> Response {
    let principal = match state.user_principal(&headers) {
        Ok(principal) => principal,
        Err(_) => return Redirect::to("/login?scope=user&next=%2Fportal").into_response(),
    };
    match portal_data(&state, &principal.subject) {
        Ok(data) => rnovemail_admin::portal_page(&page_context(&query), &data).into_response(),
        Err(error) => error.into_response(),
    }
}

fn portal_data(state: &AppState, email: &str) -> Result<PortalData, ApiRejection> {
    let email = EmailAddress::parse(email).map_err(|_| ApiRejection::BadRequest)?;
    let users = state.list_users()?;
    let owners = owner_lookup(&users);
    let owner = users
        .iter()
        .find(|user| user.primary_email() == &email)
        .ok_or(ApiRejection::NotFound)?;
    Ok(PortalData {
        email: email.as_str().to_string(),
        mailboxes: mailbox_rows(state.list_mailboxes()?, owner.id(), &owners),
    })
}

fn mailbox_rows(
    mailboxes: Vec<Mailbox>,
    owner_id: UserId,
    owners: &HashMap<UserId, String>,
) -> Vec<MailboxRow> {
    mailboxes
        .into_iter()
        .filter(|mailbox| mailbox.owner_id() == owner_id)
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
        .collect()
}

fn owner_lookup(users: &[User]) -> HashMap<UserId, String> {
    users
        .iter()
        .map(|user| (user.id(), user.primary_email().as_str().to_string()))
        .collect()
}

fn page_context(query: &PageQuery) -> PageContext {
    PageContext {
        lang: Lang::parse(query.lang.as_deref()),
        theme: Theme::parse(query.theme.as_deref()),
        next: "/portal".to_string(),
    }
}
