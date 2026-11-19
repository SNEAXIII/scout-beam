//! What the user domain accepts and returns on the wire.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::models::user_model::User;
use crate::shared::role::Role;

/// What a user may see about themselves.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserProfile {
    pub login: String,
    pub role: Role,
    /// A guest may only play the daily, until they sign up.
    pub guest: bool,
    pub last_login_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            login: user.login,
            role: user.role,
            guest: user.guest,
            last_login_date: user.last_login_date,
            created_at: user.created_at,
        }
    }
}

/// Mints a session for a login, creating the account on first sight. Only mounted outside production.
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct DevLoginRequest {
    /// Omit it and the server invents one; `validator` skips the rules when the field is absent.
    #[serde(default)]
    #[validate(
        length(
            min = 3,
            max = 30,
            code = "invalid_login",
            message = "A login is 3 to 30 characters long"
        ),
        custom(function = "validate_alphanumeric")
    )]
    pub login: Option<String>,

    /// Lets a test bootstrap an admin without a second, privileged endpoint.
    #[serde(default)]
    pub role: Option<Role>,
}

/// Creates a password account. No `Debug`: the body carries a password.
#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct RegisterRequest {
    #[validate(
        length(
            min = 3,
            max = 30,
            code = "invalid_login",
            message = "A login is 3 to 30 characters long"
        ),
        custom(function = "validate_alphanumeric")
    )]
    pub login: String,

    /// Its length is checked by the service, so a short one answers 400 rather than 422.
    pub password: String,
}

/// Signs in with a login and password. No `Debug`: the body carries a password.
#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

/// The pseudo the caller wants to go by.
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct RenameRequest {
    #[validate(
        length(
            min = 3,
            max = 30,
            message = "Le nom d'utilisateur doit faire entre 3 et 30 caractères"
        ),
        custom(function = "validate_alphanumeric")
    )]
    pub login: String,
}

/// The code a provider handed the front on its callback.
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct ProviderCodeRequest {
    #[validate(length(
        min = 1,
        code = "missing_provider_code",
        message = "The provider code is required"
    ))]
    pub code: String,
}

/// What the sign-in page may offer.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProvidersResponse {
    /// Whether `POST /auth/dev-login` is mounted.
    pub dev_login: bool,
    pub providers: Vec<ProviderLink>,
}

/// An enabled provider; the front appends its own `state` to `authorize_url`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProviderLink {
    #[schema(example = "discord")]
    pub name: &'static str,
    pub authorize_url: String,
}

/// `^[a-zA-Z0-9]+$` without pulling in a regex engine for one rule.
fn validate_alphanumeric(value: &str) -> Result<(), ValidationError> {
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric())
    {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_login")
            .with_message("A login holds letters and digits only".into()))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn rejects_non_alphanumeric_logins() {
        assert!(validate_alphanumeric("Sneaxiii").is_ok());
        assert!(validate_alphanumeric("Sne axi").is_err());
        assert!(validate_alphanumeric("Snéaxiii").is_err());
        assert!(validate_alphanumeric("drop-table").is_err());
    }
}
