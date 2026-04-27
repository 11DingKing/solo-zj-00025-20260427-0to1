use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    
    #[error("Bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
    
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(e) => {
                tracing::error!("Database error: {}", e);
                match e {
                    sqlx::Error::RowNotFound => (
                        StatusCode::NOT_FOUND,
                        "Resource not found".to_string(),
                    ),
                    sqlx::Error::Database(db_err) => {
                        if let Some(constraint) = db_err.constraint() {
                            if constraint.contains("unique") {
                                return (
                                    StatusCode::CONFLICT,
                                    "Resource already exists".to_string(),
                                )
                                .into_response();
                            }
                        }
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Database error".to_string(),
                        )
                    }
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Database error".to_string(),
                    ),
                }
            }
            AppError::Redis(e) => {
                tracing::error!("Redis error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Cache error".to_string(),
                )
            }
            AppError::Jwt(e) => {
                tracing::debug!("JWT error: {}", e);
                (
                    StatusCode::UNAUTHORIZED,
                    "Invalid or expired token".to_string(),
                )
            }
            AppError::Bcrypt(e) => {
                tracing::error!("Bcrypt error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Authentication error".to_string(),
                )
            }
            AppError::Validation(e) => {
                tracing::debug!("Validation error: {}", e);
                (StatusCode::BAD_REQUEST, e.to_string())
            }
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(e) => {
                tracing::error!("Internal error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
