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

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        std::future::ready(identity(parts, &JwtCodec::from_ref(state)).map_err(ApiError::from))
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
