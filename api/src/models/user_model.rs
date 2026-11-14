//! The `user` row.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::shared::role::Role;

/// `Debug` by hand: it says whether a password hash is set, never what it is.
#[derive(Clone, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub login: String,
    pub role: Role,
    /// Never serialised: the hash is for the sign-in check only.
    #[serde(skip)]
    pub password_hash: Option<String>,
    /// A player without credentials, until they sign up.
    pub guest: bool,
    pub last_login_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl fmt::Debug for User {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("User")
            .field("id", &self.id)
            .field("login", &self.login)
            .field("role", &self.role)
            .field("has_password", &self.password_hash.is_some())
            .field("guest", &self.guest)
            .field("last_login_date", &self.last_login_date)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_never_prints_the_password_hash() {
        let user = User {
            id: Uuid::nil(),
            login: "Sneaxiii".to_owned(),
            role: Role::User,
            password_hash: Some("$argon2id$secret".to_owned()),
            guest: false,
            last_login_date: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let printed = format!("{user:?}");
        assert!(!printed.contains("argon2id"), "{printed}");
        assert!(printed.contains("has_password: true"), "{printed}");
    }
}
