//! User-domain failures, phrased in domain vocabulary — no HTTP here but the mapping at the bottom.

use crate::shared::error::{ApiError, Reason};

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("This account does not exist")]
    DoesNotExist,

    #[error("This login is already taken")]
    LoginAlreadyTaken,

    #[error("A login is 3 to 30 characters long, letters and digits only")]
    InvalidLogin,

    #[error("A password is 8 to 128 characters long")]
    InvalidPassword,

    /// The same answer for an unknown login, an account without a password and a wrong password.
    #[error("Wrong login or password")]
    InvalidCredentials,

    #[error("This sign-in provider is not enabled")]
    ProviderDisabled,

    /// A refused code or an unreachable provider: what it answered goes to the logs only.
    #[error("Signing in with this provider failed")]
    ProviderRefused,

    /// The account with this email already holds another id at the same provider.
    #[error("This account is already linked to another one at this provider")]
    AlreadyLinked,

    /// A hashing failure: nothing the caller can act on.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),

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
        if is_unique_violation_on(&error, "login") {
            Self::LoginAlreadyTaken
        } else {
            Self::Database(error)
        }
    }
}

/// A write lost a race against another one on a unique column.
pub fn is_unique_violation(error: &sqlx::Error) -> bool {
    is_unique_violation_on(error, "")
}

fn is_unique_violation_on(error: &sqlx::Error, column: &str) -> bool {
    error.as_database_error().is_some_and(|database_error| {
        database_error.code().as_deref() == Some(UNIQUE_VIOLATION)
            && database_error
                .constraint()
                .is_some_and(|constraint| constraint.contains(column))
    })
}

impl From<UserError> for ApiError {
    fn from(error: UserError) -> Self {
        // Rendered before the match: the `Database` arm moves out of `error`.
        let reason = Reason::new(error.code(), error.to_string());
        match error {
            UserError::Database(cause) => Self::Database(cause),
            // The caller's own account cannot be used: an authentication answer.
            UserError::DoesNotExist
            | UserError::InvalidCredentials
            | UserError::ProviderRefused => Self::Unauthorized(reason),
            UserError::LoginAlreadyTaken | UserError::AlreadyLinked => Self::Conflict(reason),
            UserError::InvalidLogin | UserError::InvalidPassword => Self::BadRequest(reason),
            UserError::ProviderDisabled => Self::NotFound(reason),
            UserError::Internal(cause) => Self::Internal(cause),
        }
    }
}

impl UserError {
    /// The code the front translates.
    const fn code(&self) -> &'static str {
        match self {
            Self::DoesNotExist => "account_not_found",
            Self::LoginAlreadyTaken => "login_taken",
            Self::InvalidLogin => "invalid_login",
            Self::InvalidPassword => "invalid_password",
            Self::InvalidCredentials => "invalid_credentials",
            Self::ProviderDisabled => "provider_disabled",
            Self::ProviderRefused => "provider_refused",
            Self::AlreadyLinked => "already_linked",
            Self::Internal(_) | Self::Database(_) => "internal",
        }
    }
}
