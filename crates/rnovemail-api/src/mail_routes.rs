use axum::{
    Json, Router,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use rnovemail_domain::{EmailAddress, MessageId, ProviderAccount, ProviderType};
use rnovemail_providers::SendMailRequest;
use serde::{Deserialize, Serialize};

use crate::{AppState, middleware::ApiRejection};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/mail/send", post(send_mail))
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

async fn get_outbound(State(state): State<AppState>, headers: HeaderMap) -> Response {
    token_accept(&state, headers, "outbound_visible")
}

async fn get_inbound(State(state): State<AppState>, headers: HeaderMap) -> Response {
    token_accept(&state, headers, "inbound_visible")
}

fn token_accept(state: &AppState, headers: HeaderMap, accepted: &'static str) -> Response {
    if let Err(rejection) = state.require_admin(&headers) {
        return rejection.into_response();
    }
    axum::Json(serde_json::json!({ "status": accepted })).into_response()
}

async fn send_mail_response(
    state: &AppState,
    request: SendMailApiRequest,
) -> Result<Json<SendMailResponse>, ApiRejection> {
    let mail = request.try_into_mail_request()?;
    let (provider, receipt) = state.send_mail(mail).await?;
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
