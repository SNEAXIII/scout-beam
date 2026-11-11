//! Turning a bearer token into a caller identity.
//!
//! Database-free on purpose: whether the account still exists is the user domain's business.

use axum::extract::{FromRef, FromRequestParts};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::auth::error::AuthError;
use crate::auth::jwt::JwtCodec;
use crate::shared::error::ApiError;
use crate::shared::role::Role;

/// A verified access token, reduced to what authorization needs.
#[derive(Debug, Clone, Copy)]
pub struct Identity {
    pub user_id: Uuid,
    pub role: Role,
}

impl<S> FromRequestParts<S> for Identity
where
    JwtCodec: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let codec = JwtCodec::from_ref(state);
        let token = bearer_token(parts)?;

        let claims = codec.decode(token)?;
        Ok(Self {
            user_id: claims.subject()?,
            role: claims.access_role()?,
        })
    }
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
