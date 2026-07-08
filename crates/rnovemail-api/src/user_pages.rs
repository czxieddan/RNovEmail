use std::collections::{HashMap, HashSet};

use axum::{
    Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use rnovemail_admin::{
    Lang, MailboxRow, MessageAttachmentRow, MessageDetailRow, MessageHeaderRow, MessageRow,
    PageContext, PortalData, PortalMessageData, Theme,
};
use rnovemail_domain::{
    EmailAddress, InboundMessage, Mailbox, MailboxId, OutboundMessage, User, UserId,
};
use serde::Deserialize;

use crate::{AppState, middleware::ApiRejection};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/portal", get(portal))
        .route("/portal/inbound/{id}", get(inbound_message))
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
        Err(_) => return Redirect::to(&login_location("/portal")).into_response(),
    };
    match portal_data(&state, &principal.subject) {
        Ok(data) => {
            rnovemail_admin::portal_page(&page_context(&query, "/portal"), &data).into_response()
        }
        Err(error) => error.into_response(),
    }
}

async fn inbound_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<PageQuery>,
) -> Response {
    let next = format!("/portal/inbound/{id}");
    let principal = match state.user_principal(&headers) {
        Ok(principal) => principal,
        Err(_) => return Redirect::to(&login_location(&next)).into_response(),
    };
    match portal_message_data(&state, &principal.subject, &id).await {
        Ok(data) => rnovemail_admin::portal_message_page(&page_context(&query, &next), &data)
            .into_response(),
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

async fn portal_message_data(
    state: &AppState,
    email: &str,
    message_id: &str,
) -> Result<PortalMessageData, ApiRejection> {
    let email = EmailAddress::parse(email).map_err(|_| ApiRejection::BadRequest)?;
    let users = state.list_users()?;
    let owner = users
        .iter()
        .find(|user| user.primary_email() == &email)
        .ok_or(ApiRejection::NotFound)?;
    let mailboxes = state.list_mailboxes()?;
    let owned_mailboxes = owned_mailbox_lookup(&mailboxes, owner.id());
    let message = state.inbound_message_by_id(message_id)?;
    let mailbox = owned_message_mailbox(&message, &owned_mailboxes)?;
    let view = state.hydrate_inbound_message_view(message).await;
    let message = view.message;
    let detail_error = view.detail_error;
    Ok(PortalMessageData {
        email: email.as_str().to_string(),
        message: message_detail_row(message, mailbox, detail_error.as_deref()),
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
        id: serialized_key(&message.id),
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
        id: serialized_key(&message.id),
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

fn owned_message_mailbox(
    message: &InboundMessage,
    mailboxes: &HashMap<MailboxId, String>,
) -> Result<String, ApiRejection> {
    mailboxes
        .get(&message.mailbox_id)
        .cloned()
        .ok_or(ApiRejection::NotFound)
}

fn message_detail_row(
    message: InboundMessage,
    mailbox: String,
    detail_error: Option<&str>,
) -> MessageDetailRow {
    let detail = message.detail.as_ref();
    MessageDetailRow {
        mailbox: mailbox.clone(),
        from: detail_text(
            detail.map(|detail| detail.from.as_str()),
            message.from.as_str(),
        ),
        to: detail_list(detail.map(|detail| detail.to.as_slice()), &mailbox),
        cc: detail_list(detail.map(|detail| detail.cc.as_slice()), ""),
        bcc: detail_list(detail.map(|detail| detail.bcc.as_slice()), ""),
        reply_to: detail_list(detail.map(|detail| detail.reply_to.as_slice()), ""),
        subject: detail_text(
            detail.map(|detail| detail.subject.as_str()),
            &message.subject,
        ),
        text: detail_text(detail.map(|detail| detail.text.as_str()), &message.text),
        html: detail
            .and_then(|detail| detail.html.clone())
            .unwrap_or_default(),
        detail_error: detail_error.unwrap_or_default().to_string(),
        detail_loaded: detail.is_some(),
        received_at: message.received_at.to_rfc3339(),
        headers: detail.map(header_rows).unwrap_or_default(),
        attachments: detail.map(attachment_rows).unwrap_or_default(),
        raw_download_url: raw_download_url(detail),
        raw_expires_at: raw_expires_at(detail),
    }
}

fn detail_text(value: Option<&str>, fallback: &str) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn detail_list(values: Option<&[String]>, fallback: &str) -> String {
    values
        .map(join_values)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn join_values(values: &[String]) -> String {
    values
        .iter()
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

fn header_rows(detail: &rnovemail_domain::InboundMessageDetail) -> Vec<MessageHeaderRow> {
    detail
        .headers
        .iter()
        .map(|header| MessageHeaderRow {
            name: header.name.clone(),
            value: header.value.clone(),
        })
        .collect()
}

fn attachment_rows(detail: &rnovemail_domain::InboundMessageDetail) -> Vec<MessageAttachmentRow> {
    detail
        .attachments
        .iter()
        .map(|attachment| MessageAttachmentRow {
            filename: attachment.filename.clone(),
            content_type: attachment.content_type.clone(),
            content_disposition: attachment.content_disposition.clone(),
            content_id: attachment.content_id.clone().unwrap_or_default(),
        })
        .collect()
}

fn raw_download_url(detail: Option<&rnovemail_domain::InboundMessageDetail>) -> String {
    detail
        .and_then(|detail| detail.raw.as_ref())
        .map(|raw| raw.download_url.clone())
        .unwrap_or_default()
}

fn raw_expires_at(detail: Option<&rnovemail_domain::InboundMessageDetail>) -> String {
    detail
        .and_then(|detail| detail.raw.as_ref())
        .and_then(|raw| raw.expires_at.clone())
        .unwrap_or_default()
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

fn page_context(query: &PageQuery, next: &str) -> PageContext {
    PageContext {
        lang: Lang::parse(query.lang.as_deref()),
        theme: Theme::parse(query.theme.as_deref()),
        next: next.to_string(),
    }
}

fn login_location(next: &str) -> String {
    format!("/login?scope=user&next={}", encoded_next(next))
}

fn encoded_next(path: &str) -> String {
    path.replace('/', "%2F")
}

fn serialized_key<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| serde_json::to_string(value).unwrap_or_else(|_| "unknown".to_string()))
}
