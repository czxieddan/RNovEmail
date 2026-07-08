use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use rnovemail_domain::{
    EmailAddress, InboundMessage, InboundMessageDetail, MessageId, OutboundMessage,
    ProviderAccount, ProviderType,
};
use rnovemail_providers::SendMailRequest;
use serde::{Deserialize, Serialize};

use crate::{AppState, middleware::ApiRejection};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/mail/send", post(send_mail))
        .route("/api/v1/portal/mail/send", post(send_portal_mail))
        .route("/api/v1/mail/outbound/{id}", get(get_outbound))
        .route("/api/v1/mail/inbound/{id}", get(get_inbound))
}

async fn send_mail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<SendMailApiRequest>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    send_mail_response(&state, request).await.into_response()
}

async fn send_portal_mail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<SendMailApiRequest>,
) -> Response {
    let principal = match state.user_principal(&headers) {
        Ok(principal) => principal,
        Err(rejection) => return rejection.into_response(),
    };
    send_user_mail_response(&state, &principal.subject, request)
        .await
        .into_response()
}

async fn get_outbound(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    outbound_response(&state, &id).into_response()
}

async fn get_inbound(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    inbound_response(&state, &id).await.into_response()
}

async fn send_mail_response(
    state: &AppState,
    request: SendMailApiRequest,
) -> Result<Json<SendMailResponse>, ApiRejection> {
    let mail = request.try_into_mail_request()?;
    let (provider, receipt) = state.send_mail(mail).await?;
    Ok(Json(SendMailResponse::sent(provider, receipt)))
}

async fn send_user_mail_response(
    state: &AppState,
    user_email: &str,
    request: SendMailApiRequest,
) -> Result<Json<SendMailResponse>, ApiRejection> {
    let mail = request.try_into_mail_request()?;
    let (provider, receipt) = state.send_user_mail(user_email, mail).await?;
    Ok(Json(SendMailResponse::sent(provider, receipt)))
}

#[derive(Deserialize)]
struct SendMailApiRequest {
    from: String,
    to: Vec<String>,
    subject: String,
    text: String,
    html: Option<String>,
}

impl SendMailApiRequest {
    fn try_into_mail_request(self) -> Result<SendMailRequest, ApiRejection> {
        let from = parse_email(&self.from)?;
        let recipients = parse_recipients(self.to)?;
        let builder = SendMailRequest::builder()
            .from(from)
            .to(recipients)
            .subject(self.subject)
            .text(self.text);
        build_mail_request(builder, self.html)
    }
}

#[derive(Serialize)]
struct SendMailResponse {
    status: &'static str,
    provider_type: &'static str,
    message_id: MessageId,
    provider_message_id: String,
}

#[derive(Serialize)]
struct InboundMessageResponse {
    id: String,
    mailbox_id: String,
    provider_account_id: Option<String>,
    provider_event_id: String,
    from: String,
    subject: String,
    text: String,
    html: Option<String>,
    received_at: String,
    detail_available: bool,
    detail_error: Option<String>,
    detail: Option<InboundMessageDetail>,
}

#[derive(Serialize)]
struct OutboundMessageResponse {
    id: String,
    provider_account_id: String,
    from: String,
    to: Vec<String>,
    subject: String,
    text: String,
    status: String,
}

impl SendMailResponse {
    fn sent(provider: ProviderAccount, receipt: rnovemail_providers::ProviderSendReceipt) -> Self {
        Self {
            status: "sent",
            provider_type: provider_type_name(provider.provider_type()),
            message_id: receipt.message_id,
            provider_message_id: receipt.provider_message_id,
        }
    }
}

async fn inbound_response(
    state: &AppState,
    id: &str,
) -> Result<Json<InboundMessageResponse>, ApiRejection> {
    let view = state.inbound_message_view_by_id(id).await?;
    Ok(Json(InboundMessageResponse::from_view(
        view.message,
        view.detail_error,
    )))
}

fn outbound_response(
    state: &AppState,
    id: &str,
) -> Result<Json<OutboundMessageResponse>, ApiRejection> {
    let message = state.outbound_message_by_id(id)?;
    Ok(Json(OutboundMessageResponse::from_message(message)))
}

impl InboundMessageResponse {
    fn from_view(message: InboundMessage, detail_error: Option<String>) -> Self {
        let detail = message.detail.clone();
        Self {
            id: serialized_key(&message.id),
            mailbox_id: serialized_key(&message.mailbox_id),
            provider_account_id: message.provider_account_id.map(|id| serialized_key(&id)),
            provider_event_id: message.provider_event_id,
            from: message.from.as_str().to_string(),
            subject: message.subject,
            text: message.text,
            html: detail.as_ref().and_then(|detail| detail.html.clone()),
            received_at: message.received_at.to_rfc3339(),
            detail_available: detail.is_some(),
            detail_error,
            detail,
        }
    }
}

impl OutboundMessageResponse {
    fn from_message(message: OutboundMessage) -> Self {
        Self {
            id: serialized_key(&message.id),
            provider_account_id: serialized_key(&message.provider_account_id),
            from: message.from.as_str().to_string(),
            to: message
                .to
                .iter()
                .map(|email| email.as_str().to_string())
                .collect(),
            subject: message.subject,
            text: message.text,
            status: format!("{:?}", message.status),
        }
    }
}

fn parse_email(value: &str) -> Result<EmailAddress, ApiRejection> {
    EmailAddress::parse(value).map_err(|_| ApiRejection::BadRequest)
}

fn parse_recipients(values: Vec<String>) -> Result<Vec<EmailAddress>, ApiRejection> {
    values
        .into_iter()
        .map(|value| EmailAddress::parse(value).map_err(|_| ApiRejection::BadRequest))
        .collect()
}

fn build_mail_request(
    builder: rnovemail_providers::SendMailRequestBuilder,
    html: Option<String>,
) -> Result<SendMailRequest, ApiRejection> {
    let builder = match html {
        Some(value) => builder.html(value),
        None => builder,
    };
    builder.build().map_err(|_| ApiRejection::BadRequest)
}

fn provider_type_name(provider_type: ProviderType) -> &'static str {
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
