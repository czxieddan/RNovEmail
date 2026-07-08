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
        json_error(self.status(), self.code())
    }
}

impl ApiRejection {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::MissingApiToken => "missing_api_token",
            Self::InvalidApiToken => "invalid_api_token",
            Self::BadRequest => "bad_request",
            Self::NotFound => "not_found",
            Self::NoProviderForDomain => "no_provider_for_domain",
            Self::ProviderApiKeyMissing => "provider_api_key_missing",
            Self::ProviderRejected => "provider_rejected",
            Self::MailboxAccessDenied => "mailbox_access_denied",
            Self::InvalidWebhookSignature => "invalid_webhook_signature",
            Self::DuplicateWebhookEvent => "duplicate_webhook_event",
            Self::StateUnavailable => "state_unavailable",
            Self::TooManyLoginAttempts => "too_many_login_attempts",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::MissingApiToken | Self::InvalidApiToken | Self::InvalidWebhookSignature => {
                StatusCode::UNAUTHORIZED
            }
            Self::BadRequest | Self::ProviderApiKeyMissing => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::NoProviderForDomain | Self::MailboxAccessDenied => StatusCode::FORBIDDEN,
            Self::ProviderRejected => StatusCode::BAD_GATEWAY,
            Self::DuplicateWebhookEvent => StatusCode::CONFLICT,
            Self::StateUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::TooManyLoginAttempts => StatusCode::TOO_MANY_REQUESTS,
        }
    }
}

pub fn json_error(status: StatusCode, code: &'static str) -> Response {
    (status, Json(ErrorBody { error: code })).into_response()
}
