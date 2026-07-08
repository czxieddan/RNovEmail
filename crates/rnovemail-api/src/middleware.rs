use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

pub enum ApiRejection {
    MissingApiToken,
    InvalidApiToken,
    BadRequest,
    NotFound,
    NoProviderForDomain,
    ProviderApiKeyMissing,
    ProviderRejected { status: Option<u16> },
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
    pub(crate) fn code(&self) -> String {
        match self {
            Self::MissingApiToken => "missing_api_token".to_string(),
            Self::InvalidApiToken => "invalid_api_token".to_string(),
            Self::BadRequest => "bad_request".to_string(),
            Self::NotFound => "not_found".to_string(),
            Self::NoProviderForDomain => "no_provider_for_domain".to_string(),
            Self::ProviderApiKeyMissing => "provider_api_key_missing".to_string(),
            Self::ProviderRejected { status } => provider_rejected_code(*status),
            Self::MailboxAccessDenied => "mailbox_access_denied".to_string(),
            Self::InvalidWebhookSignature => "invalid_webhook_signature".to_string(),
            Self::DuplicateWebhookEvent => "duplicate_webhook_event".to_string(),
            Self::StateUnavailable => "state_unavailable".to_string(),
            Self::TooManyLoginAttempts => "too_many_login_attempts".to_string(),
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
            Self::ProviderRejected { .. } => StatusCode::BAD_GATEWAY,
            Self::DuplicateWebhookEvent => StatusCode::CONFLICT,
            Self::StateUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::TooManyLoginAttempts => StatusCode::TOO_MANY_REQUESTS,
        }
    }
}

fn provider_rejected_code(status: Option<u16>) -> String {
    match status {
        Some(status) => format!("provider_rejected_{status}"),
        None => "provider_rejected".to_string(),
    }
}

pub fn json_error(status: StatusCode, code: String) -> Response {
    (status, Json(ErrorBody { error: code })).into_response()
}
