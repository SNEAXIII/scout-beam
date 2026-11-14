//! Turning an access token into a caller identity: the `Authorization: Bearer` header first
//! (tests, tooling), else the access cookie the browser sends.
//!
//! Database-free on purpose: whether the account still exists is the user domain's business.

use std::convert::Infallible;

use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::HeaderValue;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::auth::cookie::{self, ACCESS_COOKIE};
use crate::auth::error::AuthError;
use crate::auth::jwt::JwtCodec;
use crate::shared::error::ApiError;
use crate::shared::role::Role;
use crate::state::AppState;

/// A verified access token, reduced to what authorization needs.
#[derive(Debug, Clone, Copy)]
pub struct Identity {
    pub user_id: Uuid,
    pub role: Role,
    /// The token's `exp`, in seconds since the epoch: a socket closes on it.
    pub expires_at: i64,
}

impl FromRequestParts<AppState> for Identity {
    type Rejection = ApiError;

    #[expect(
        clippy::unused_async_trait_impl,
        reason = "axum's trait is async; checking a JWT awaits nothing"
    )]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        identity(parts, &state.jwt).map_err(ApiError::from)
    }
}

/// `Option<Identity>`: for routes open to anyone, where a missing, invalid or expired token
/// counts as no caller at all rather than a 401.
impl OptionalFromRequestParts<AppState> for Identity {
    type Rejection = Infallible;

    #[expect(
        clippy::unused_async_trait_impl,
        reason = "axum's trait is async; checking a JWT awaits nothing"
    )]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Option<Self>, Self::Rejection> {
        Ok(identity(parts, &state.jwt).ok())
    }
}

fn identity(parts: &Parts, codec: &JwtCodec) -> Result<Identity, AuthError> {
    let claims = codec.decode_access(access_token(parts)?)?;
    Ok(Identity {
        user_id: claims.sub,
        role: claims.role,
        expires_at: claims.exp,
    })
}

/// A header that is there but malformed is refused rather than skipped for the cookie.
fn access_token(parts: &Parts) -> Result<&str, AuthError> {
    parts.headers.get(AUTHORIZATION).map_or_else(
        || cookie::read(&parts.headers, ACCESS_COOKIE).ok_or(AuthError::MissingToken),
        bearer_token,
    )
}

fn bearer_token(header: &HeaderValue) -> Result<&str, AuthError> {
    header
        .to_str()
        .map_err(|_| AuthError::InvalidToken)?
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .ok_or(AuthError::InvalidToken)
}
