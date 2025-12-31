//! API error types and conversions.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// API error type that maps to HTTP status codes
#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    InternalError(String),
    DaemonNotRunning,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
            ApiError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg)
            }
            ApiError::DaemonNotRunning => (
                StatusCode::SERVICE_UNAVAILABLE,
                "DAEMON_NOT_RUNNING",
                "Daemon is not running".to_string(),
            ),
        };

        let body = Json(json!({
            "success": false,
            "error": {
                "code": code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

// Convenience conversions
impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        ApiError::InternalError(e.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for ApiError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        ApiError::InternalError(e.to_string())
    }
}
