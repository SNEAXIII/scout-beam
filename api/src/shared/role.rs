//! Authorization vocabulary.
//!
//! In `shared` rather than the user domain: every domain reads it, and it travels in the JWT.

use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    sqlx::Type,
    utoipa::ToSchema,
)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Ordered from least to most privileged.
    User,
    Admin,
    SuperAdmin,
}

impl Role {
    /// The spelling used by the database, the JWT and the API.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Admin => "admin",
            Self::SuperAdmin => "super_admin",
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn as_str_matches_the_serialized_form() {
        for role in [Role::User, Role::Admin, Role::SuperAdmin] {
            let serialized = serde_json::to_string(&role).unwrap();
            assert_eq!(format!("\"{}\"", role.as_str()), serialized);
        }
    }

    #[test]
    fn serializes_in_snake_case() {
        assert_eq!(
            serde_json::to_string(&Role::SuperAdmin).unwrap(),
            "\"super_admin\""
        );
        assert_eq!(serde_json::to_string(&Role::User).unwrap(), "\"user\"");
    }
}
