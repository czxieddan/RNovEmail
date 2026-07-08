use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
}

pub enum ApiRejection {
    MissingApiToken,
    InvalidApiToken,
    BadRequest,
    NotFound,
    NoProviderForDomain,
    ProviderApiKeyMissing,
    ProviderRejected,
    MailboxAccessDenied,
    InvalidWebhookSignature,
    DuplicateWebhookEvent,
    StateUnavailable,
    TooManyLoginAttempts,
}

impl IntoResponse for ApiRejection {
    fn into_response(self) -> Response {
        match self {
            Self::MissingApiToken => json_error(StatusCode::UNAUTHORIZED, "missing_api_token"),
            Self::InvalidApiToken => json_error(StatusCode::UNAUTHORIZED, "invalid_api_token"),
            Self::BadRequest => json_error(StatusCode::BAD_REQUEST, "bad_request"),
            Self::NotFound => json_error(StatusCode::NOT_FOUND, "not_found"),
            Self::NoProviderForDomain => {
                json_error(StatusCode::FORBIDDEN, "no_provider_for_domain")
            }
            Self::ProviderApiKeyMissing => {
                json_error(StatusCode::BAD_REQUEST, "provider_api_key_missing")
            }
            Self::ProviderRejected => json_error(StatusCode::BAD_GATEWAY, "provider_rejected"),
            Self::MailboxAccessDenied => json_error(StatusCode::FORBIDDEN, "mailbox_access_denied"),
            Self::InvalidWebhookSignature => {
                json_error(StatusCode::UNAUTHORIZED, "invalid_webhook_signature")
            }
            Self::DuplicateWebhookEvent => {
                json_error(StatusCode::CONFLICT, "duplicate_webhook_event")
            }
            Self::StateUnavailable => {
                json_error(StatusCode::SERVICE_UNAVAILABLE, "state_unavailable")
            }
            Self::TooManyLoginAttempts => {
                json_error(StatusCode::TOO_MANY_REQUESTS, "too_many_login_attempts")
            }
        }
    }
}

pub fn json_error(status: StatusCode, code: &'static str) -> Response {
    (status, Json(ErrorBody { error: code })).into_response()
}
