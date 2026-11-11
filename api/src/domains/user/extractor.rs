//! Handler arguments that guarantee an authenticated account that still exists.
//!
//! In the user domain, not in `auth`, because reading the `user` table is this domain's job.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::auth::Identity;
use crate::domains::user::model::User;
use crate::domains::user::service;
use crate::shared::error::ApiError;
use crate::state::AppState;

/// An authenticated user whose account still exists.
#[derive(Debug, Clone)]
pub struct CurrentUser(pub User);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let identity = Identity::from_request_parts(parts, state).await?;
        Ok(Self(
            service::load_active(&state.pool, identity.user_id).await?,
        ))
    }
}
