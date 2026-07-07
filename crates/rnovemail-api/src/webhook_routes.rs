use axum::{
    Router,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};

use crate::{AppState, middleware};

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/v1/webhooks/{provider}/{account_id}", post(ingest))
}

async fn ingest(headers: HeaderMap) -> Response {
    match headers.get("x-rnovemail-signature") {
        Some(_) => axum::Json(serde_json::json!({ "status": "webhook_accepted" })).into_response(),
        None => middleware::json_error(StatusCode::UNAUTHORIZED, "invalid_webhook_signature"),
    }
}
