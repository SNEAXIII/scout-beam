//! User-domain failures, phrased in domain vocabulary — no HTTP here but the mapping at the bottom.

use crate::shared::error::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("Ce compte n'existe pas")]
    DoesNotExist,

    #[error("Ce pseudo est déjà pris")]
    LoginAlreadyTaken,

    #[error(
        "Le nom d'utilisateur doit faire entre 3 et 30 caractères et ne contenir que des chiffres et des lettres"
    )]
    InvalidLogin,

    /// Kept only so services can `?` on repository calls; handed straight back to `ApiError`.
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

/// PostgreSQL's `unique_violation`.
const UNIQUE_VIOLATION: &str = "23505";

impl UserError {
    /// Read the business meaning out of a failed write: the unique index is the real
    /// guarantee against a login taken between the pre-check and the insert.
    pub fn from_write(error: sqlx::Error) -> Self {
        if is_login_conflict(&error) {
            Self::LoginAlreadyTaken
        } else {
            Self::Database(error)
        }
    }
}

fn is_login_conflict(error: &sqlx::Error) -> bool {
    error.as_database_error().is_some_and(|database_error| {
        database_error.code().as_deref() == Some(UNIQUE_VIOLATION)
            && database_error
                .constraint()
                .is_some_and(|constraint| constraint.contains("login"))
    })
}

impl From<UserError> for ApiError {
    fn from(error: UserError) -> Self {
        // Rendered before the match: the `Database` arm moves out of `error`.
        let message = error.to_string();
        match error {
            UserError::Database(cause) => Self::Database(cause),
            // The caller's own account cannot be used: an authentication answer.
            UserError::DoesNotExist => Self::Unauthorized(message),
            UserError::LoginAlreadyTaken => Self::Conflict(message),
            UserError::InvalidLogin => Self::BadRequest(message),
        }
    }
}
