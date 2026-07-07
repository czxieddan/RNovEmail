use axum::{
    Router,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::{get, post},
};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/mail/send", post(send_mail))
        .route("/api/v1/mail/outbound/{id}", get(get_outbound))
        .route("/api/v1/mail/inbound/{id}", get(get_inbound))
}

async fn send_mail(State(state): State<AppState>, headers: HeaderMap) -> Response {
    token_accept(&state, headers, "send_accepted")
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
