//! The single place that knows how a failure becomes an HTTP response.
//!
//! Domains raise their own error enums and convert into `ApiError` at the edge.

use std::borrow::Cow;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::{Value, json};
use validator::ValidationErrors;

/// What a refusal tells the player: a stable `code` the front translates, filled from `params`,
/// and an English `message` for any other client.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct Reason {
    pub code: &'static str,
    pub params: Value,
    pub message: String,
}

impl Reason {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            params: Value::Null,
            message: message.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Not authenticated, or authenticated as an account that may not be used.
    #[error("{0}")]
    Unauthorized(Reason),

    /// The request is well-formed but breaks a business rule.
    #[error("{0}")]
    BadRequest(Reason),

    /// Authenticated, but this account may not do that.
    #[error("{0}")]
    Forbidden(Reason),

    /// The addressed thing does not exist.
    #[error("{0}")]
    NotFound(Reason),

    /// The request conflicts with the current state (a taken login, …).
    #[error("{0}")]
    Conflict(Reason),

    /// A conflict that lifts by itself: the body also says when to try again.
    #[error("{reason}")]
    RetryLater { reason: Reason, retry_after_ms: u64 },

    /// The payload did not pass field validation.
    #[error("validation failed")]
    Validation(#[from] ValidationErrors),

    /// The body could not be parsed at all.
    #[error("{0}")]
    UnprocessableBody(Reason),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

/// Every error answers `{code, detail}`, plus `params` to fill the code's text and
/// `retry_after_ms` when it lifts by itself.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    pub code: Cow<'static, str>,
    #[serde(skip_serializing_if = "Value::is_null")]
    pub params: Value,
    pub detail: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<u64>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = self.into_parts();
        (status, Json(body)).into_response()
    }
}

impl ApiError {
    /// The status and body an HTTP response carries; a WebSocket error carries them too.
    pub fn into_parts(self) -> (StatusCode, ErrorBody) {
        let (status, reason, retry_after_ms) = match self {
            Self::Unauthorized(reason) => (StatusCode::UNAUTHORIZED, reason, None),
            Self::BadRequest(reason) => (StatusCode::BAD_REQUEST, reason, None),
            Self::Forbidden(reason) => (StatusCode::FORBIDDEN, reason, None),
            Self::NotFound(reason) => (StatusCode::NOT_FOUND, reason, None),
            Self::Conflict(reason) => (StatusCode::CONFLICT, reason, None),
            Self::RetryLater {
                reason,
                retry_after_ms,
            } => (StatusCode::CONFLICT, reason, Some(retry_after_ms)),
            Self::UnprocessableBody(reason) => (StatusCode::UNPROCESSABLE_ENTITY, reason, None),
            Self::Validation(errors) => {
                let body = ErrorBody {
                    code: validation_code(&errors),
                    params: Value::Null,
                    detail: json!(validation_details(&errors)),
                    retry_after_ms: None,
                };
                return (StatusCode::UNPROCESSABLE_ENTITY, body);
            }
            // Nothing the caller can act on: log the cause, never leak SQL.
            Self::Database(error) => {
                tracing::error!(error = %error, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, internal(), None)
            }
            Self::Internal(error) => {
                tracing::error!(error = ?error, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, internal(), None)
            }
        };
        let body = ErrorBody {
            code: Cow::Borrowed(reason.code),
            params: reason.params,
            detail: json!(reason.message),
            retry_after_ms,
        };
        (status, body)
    }
}

fn internal() -> Reason {
    Reason::new("internal", "Internal server error")
}

/// The first failed rule's code, as `validator` names it.
fn validation_code(errors: &ValidationErrors) -> Cow<'static, str> {
    errors
        .field_errors()
        .into_values()
        .flatten()
        .next()
        .map_or(Cow::Borrowed("invalid_body"), |error| error.code.clone())
}

/// Flatten `validator`'s nested map into a `[{loc, msg, type}]` list.
fn validation_details(errors: &ValidationErrors) -> Vec<Value> {
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
