//! The single place that knows how a failure becomes an HTTP response.
//!
//! Domains raise their own error enums and convert into `ApiError` at the edge.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;
use validator::ValidationErrors;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Not authenticated, or authenticated as an account that may not be used.
    #[error("{0}")]
    Unauthorized(String),

    /// The request is well-formed but breaks a business rule.
    #[error("{0}")]
    BadRequest(String),

    /// The addressed thing does not exist.
    #[error("{0}")]
    NotFound(String),

    /// The request conflicts with the current state (a taken login, …).
    #[error("{0}")]
    Conflict(String),

    /// The payload did not pass field validation.
    #[error("validation failed")]
    Validation(#[from] ValidationErrors),

    /// The body could not be parsed at all.
    #[error("{0}")]
    UnprocessableBody(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

/// Every error answers `{"detail": ...}`.
#[derive(Debug, Serialize)]
struct ErrorBody<T> {
    detail: T,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, detail) = match self {
            Self::Unauthorized(message) => (StatusCode::UNAUTHORIZED, json!(message)),
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, json!(message)),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, json!(message)),
            Self::Conflict(message) => (StatusCode::CONFLICT, json!(message)),
            Self::UnprocessableBody(message) => (StatusCode::UNPROCESSABLE_ENTITY, json!(message)),
            Self::Validation(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!(validation_details(&errors)),
            ),
            // Nothing the caller can act on: log the cause, never leak SQL.
            Self::Database(error) => {
                tracing::error!(error = %error, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!("Erreur interne du serveur"),
                )
            }
            Self::Internal(error) => {
                tracing::error!(error = ?error, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!("Erreur interne du serveur"),
                )
            }
        };

        (status, Json(ErrorBody { detail })).into_response()
    }
}

/// Flatten `validator`'s nested map into a `[{loc, msg, type}]` list.
fn validation_details(errors: &ValidationErrors) -> Vec<serde_json::Value> {
    errors
        .field_errors()
        .iter()
        .flat_map(|(field, field_errors)| {
            field_errors.iter().map(move |error| {
                json!({
                    "loc": ["body", field],
                    "msg": error
                        .message
                        .as_ref().map_or_else(|| error.code.to_string(), ToString::to_string),
                    "type": error.code,
                })
            })
        })
        .collect()
}

pub type ApiResult<T> = Result<T, ApiError>;
