//! What the user domain accepts and returns on the wire.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::domains::user::model::User;
use crate::shared::role::Role;

/// What a user may see about themselves.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserProfile {
    pub login: String,
    pub role: Role,
    pub last_login_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            login: user.login,
            role: user.role,
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
            message = "Le nom d'utilisateur doit faire entre 3 et 30 caractères"
        ),
        custom(function = "validate_alphanumeric")
    )]
    pub login: Option<String>,

    /// Lets a test bootstrap an admin without a second, privileged endpoint.
    #[serde(default)]
    pub role: Option<Role>,
}

/// `^[a-zA-Z0-9]+$` without pulling in a regex engine for one rule.
pub fn validate_alphanumeric(value: &str) -> Result<(), ValidationError> {
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric())
    {
        Ok(())
    } else {
        Err(ValidationError::new("login_non_alphanum").with_message(
            "Le nom d'utilisateur ne doit contenir que des chiffres et des lettres".into(),
        ))
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
