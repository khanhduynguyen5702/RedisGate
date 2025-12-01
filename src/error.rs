// Centralized error handling for RedisGate
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tracing::{error, warn, debug};

/// Application-wide error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Kubernetes error: {0}")]
    Kubernetes(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::Database(e) => {
                error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred")
            }
            AppError::Authentication(msg) => {
                warn!("Authentication error: {}", msg);
                (StatusCode::UNAUTHORIZED, msg.as_str())
            }
            AppError::Authorization(msg) => {
                warn!("Authorization error: {}", msg);
                (StatusCode::FORBIDDEN, msg.as_str())
            }
            AppError::Validation(msg) => {
                debug!("Validation error: {}", msg);
                (StatusCode::BAD_REQUEST, msg.as_str())
            }
            AppError::NotFound(msg) => {
                debug!("Not found: {}", msg);
                (StatusCode::NOT_FOUND, msg.as_str())
            }
            AppError::Conflict(msg) => {
                debug!("Conflict: {}", msg);
                (StatusCode::CONFLICT, msg.as_str())
            }
            AppError::Redis(msg) => {
                error!("Redis error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str())
            }
            AppError::Kubernetes(msg) => {
                error!("Kubernetes error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str())
            }
            AppError::RateLimitExceeded => {
                warn!("Rate limit exceeded");
                (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded")
            }
            AppError::Internal(msg) => {
                error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::BadRequest(msg) => {
                debug!("Bad request: {}", msg);
                (StatusCode::BAD_REQUEST, msg.as_str())
            }
        };

        let body = Json(json!({
            "success": false,
            "error": error_message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));

        (status, body).into_response()
    }
}

// Helper function to convert validation errors
pub fn validation_error(errors: validator::ValidationErrors) -> AppError {
    AppError::Validation(format!("{:?}", errors))
}

