use std::collections::{HashMap, HashSet};

use axum::{
    Router,
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use rnovemail_admin::{Lang, MailboxRow, MessageRow, PageContext, PortalData, Theme};
use rnovemail_domain::{
    EmailAddress, InboundMessage, Mailbox, MailboxId, OutboundMessage, User, UserId,
};
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
    let mailboxes = state.list_mailboxes()?;
    let owned_mailboxes = owned_mailbox_lookup(&mailboxes, owner.id());
    let owned_addresses = owned_address_lookup(&mailboxes, owner.id());
    Ok(PortalData {
        email: email.as_str().to_string(),
        mailboxes: mailbox_rows(mailboxes, owner.id(), &owners),
        inbox: inbound_rows(state.list_inbound_messages()?, &owned_mailboxes),
        sent: outbound_rows(state.list_outbound_messages()?, &owned_addresses),
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

fn inbound_rows(
    messages: Vec<InboundMessage>,
    mailboxes: &HashMap<MailboxId, String>,
) -> Vec<MessageRow> {
    messages
        .into_iter()
        .filter_map(|message| inbound_row(message, mailboxes))
        .collect()
}

fn inbound_row(
    message: InboundMessage,
    mailboxes: &HashMap<MailboxId, String>,
) -> Option<MessageRow> {
    Some(MessageRow {
        mailbox: mailboxes.get(&message.mailbox_id)?.clone(),
        from: message.from.as_str().to_string(),
        to: String::new(),
        subject: message.subject,
        text: message.text,
        status: "Received".to_string(),
        at: message.received_at.to_rfc3339(),
    })
}

fn outbound_rows(messages: Vec<OutboundMessage>, addresses: &HashSet<String>) -> Vec<MessageRow> {
    messages
        .into_iter()
        .filter(|message| addresses.contains(message.from.as_str()))
        .map(outbound_row)
        .collect()
}

fn outbound_row(message: OutboundMessage) -> MessageRow {
    MessageRow {
        mailbox: message.from.as_str().to_string(),
        from: message.from.as_str().to_string(),
        to: message
            .to
            .iter()
            .map(|email| email.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        subject: message.subject,
        text: message.text,
        status: format!("{:?}", message.status),
        at: message
            .timeline
            .last()
            .map(|entry| entry.at.to_rfc3339())
            .unwrap_or_default(),
    }
}

fn owned_mailbox_lookup(mailboxes: &[Mailbox], owner_id: UserId) -> HashMap<MailboxId, String> {
    mailboxes
        .iter()
        .filter(|mailbox| mailbox.owner_id() == owner_id)
        .map(|mailbox| (mailbox.id(), mailbox.address().as_str().to_string()))
        .collect()
}

fn owned_address_lookup(mailboxes: &[Mailbox], owner_id: UserId) -> HashSet<String> {
    mailboxes
        .iter()
        .filter(|mailbox| mailbox.owner_id() == owner_id)
        .map(|mailbox| mailbox.address().as_str().to_string())
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
