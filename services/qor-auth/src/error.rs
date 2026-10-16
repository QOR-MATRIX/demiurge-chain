//! Error handling for the Qor Auth service.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

/// Application-specific errors
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("User not found")]
    UserNotFound,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    /// The request conflicts with what is already so, such as a name already chosen; nothing was changed.
    #[error("Conflict: {0}")]
    Conflict(String),

    /// The endpoint exists but does nothing yet. It refuses rather than
    /// reporting a success it did not achieve.
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Something the request needs, such as email delivery, is not configured, so
    /// the work was not done. Refused rather than reported as done.
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// The caller has asked for this too often; nothing was done.
    #[error("Rate limited: {0}")]
    RateLimited(String),

    /// The address is marked undeliverable, after a permanent bounce or a complaint, so nothing is
    /// sent to it.
    #[error("Undeliverable: {0}")]
    Undeliverable(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    RedisError(#[from] deadpool_redis::PoolError),

    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            AppError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "INVALID_CREDENTIALS",
                "Invalid credentials".into(),
            ),
            AppError::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                "TOKEN_EXPIRED",
                "Token has expired".into(),
            ),
            AppError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "INVALID_TOKEN",
                "Invalid or malformed token".into(),
            ),
            AppError::UserNotFound => (
                StatusCode::NOT_FOUND,
                "USER_NOT_FOUND",
                "User not found".into(),
            ),
            AppError::InsufficientPermissions => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Insufficient permissions".into(),
            ),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg),
            AppError::NotImplemented(msg) => (StatusCode::NOT_IMPLEMENTED, "NOT_IMPLEMENTED", msg),
            AppError::ServiceUnavailable(msg) => {
                (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE", msg)
            }
            AppError::RateLimited(msg) => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED", msg),
            AppError::Undeliverable(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "UNDELIVERABLE", msg)
            }
            AppError::DatabaseError(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "Database error occurred".into(),
                )
            }
            AppError::RedisError(e) => {
                tracing::error!("Redis error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "CACHE_ERROR",
                    "Cache error occurred".into(),
                )
            }
            AppError::InternalError(e) => {
                tracing::error!("Internal error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error".into(),
                )
            }
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message
            }
        }));

        (status, body).into_response()
    }
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;
