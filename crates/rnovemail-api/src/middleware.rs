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
    StateUnavailable,
}

impl IntoResponse for ApiRejection {
    fn into_response(self) -> Response {
        match self {
            Self::MissingApiToken => json_error(StatusCode::UNAUTHORIZED, "missing_api_token"),
            Self::InvalidApiToken => json_error(StatusCode::UNAUTHORIZED, "invalid_api_token"),
            Self::BadRequest => json_error(StatusCode::BAD_REQUEST, "bad_request"),
            Self::NotFound => json_error(StatusCode::NOT_FOUND, "not_found"),
            Self::StateUnavailable => {
                json_error(StatusCode::SERVICE_UNAVAILABLE, "state_unavailable")
            }
        }
    }
}

pub fn json_error(status: StatusCode, code: &'static str) -> Response {
    (status, Json(ErrorBody { error: code })).into_response()
}
