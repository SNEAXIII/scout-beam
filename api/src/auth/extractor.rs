//! Turning a bearer token into a caller identity.
//!
//! Database-free on purpose: whether the account still exists is the user domain's business.

use std::convert::Infallible;

use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use uuid::Uuid;

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

/// `Option<Identity>`: for routes open to anyone, where a missing, invalid or expired bearer
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
    let claims = codec.decode_access(bearer_token(parts)?)?;
    Ok(Identity {
        user_id: claims.sub,
        role: claims.role,
    })
}

fn bearer_token(parts: &Parts) -> Result<&str, AuthError> {
    parts
        .headers
        .get(AUTHORIZATION)
        .ok_or(AuthError::MissingToken)?
        .to_str()
        .map_err(|_| AuthError::InvalidToken)?
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .ok_or(AuthError::InvalidToken)
}
