//! Wire shapes for tokens.
//!
//! Anything that needs a user row is the user domain's business, so `auth` stays a leaf.

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub token_type: &'static str,
    pub access_token: String,
    pub refresh_token: String,
}

impl LoginResponse {
    pub const fn bearer(access_token: String, refresh_token: String) -> Self {
        Self {
            token_type: "bearer",
            access_token,
            refresh_token,
        }
    }
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "Le refresh token est requis"))]
    pub refresh_token: String,
}
