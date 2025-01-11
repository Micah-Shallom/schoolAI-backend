use crate::utils::response::error_response;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    Conflict(String),
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            Self::Conflict(msg) => (StatusCode::CONFLICT, msg),
            Self::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(error_response(&message))).into_response()
    }
}
