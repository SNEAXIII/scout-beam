//! Handler arguments that guarantee an authenticated account that still exists.
//!
//! In the user domain, not in `auth`, because reading the `user` table is this domain's job.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

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
        Self::of(state, identity.user_id).await
    }
}

impl CurrentUser {
    /// The account behind a token read outside the headers, as a WebSocket's first message.
    pub async fn of(state: &AppState, user_id: Uuid) -> Result<Self, ApiError> {
        Ok(Self(service::load_active(&state.pool, user_id).await?))
    }
}
