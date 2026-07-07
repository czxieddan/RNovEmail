use axum::{
    Router,
    body::Bytes,
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::post,
};
use rnovemail_providers::ProviderWebhookRequest;

use crate::{AppState, middleware::ApiRejection};

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/v1/webhooks/{provider}/{account_id}", post(ingest))
}

async fn ingest(
    State(state): State<AppState>,
    Path((provider, account_id)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    match webhook_request(&headers, body) {
        Ok(request) => accept_webhook(&state, &provider, &account_id, request).await,
        Err(rejection) => rejection.into_response(),
    }
}

async fn accept_webhook(
    state: &AppState,
    provider: &str,
    account_id: &str,
    request: ProviderWebhookRequest,
) -> Response {
    match state.ingest_webhook(provider, account_id, request).await {
        Ok(()) => axum::Json(serde_json::json!({ "status": "webhook_accepted" })).into_response(),
        Err(rejection) => rejection.into_response(),
    }
}

fn webhook_request(
    headers: &HeaderMap,
    body: Bytes,
) -> Result<ProviderWebhookRequest, ApiRejection> {
    Ok(ProviderWebhookRequest {
        id: required_header(headers, "svix-id")?,
        timestamp: required_header(headers, "svix-timestamp")?,
        signature: required_header(headers, "svix-signature")?,
        body: body.to_vec(),
    })
}

fn required_header(headers: &HeaderMap, name: &'static str) -> Result<String, ApiRejection> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
        .ok_or(ApiRejection::InvalidWebhookSignature)
}
