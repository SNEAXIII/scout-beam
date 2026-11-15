//! Authentication failures, in authentication vocabulary.

use crate::shared::error::{ApiError, Reason};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("No token was sent")]
    MissingToken,

    #[error("The token sent is invalid")]
    InvalidToken,

    #[error("The token sent has expired")]
    ExpiredToken,

    /// A refresh token was presented where an access token is required, or the other way round.
    #[error("The token type is invalid")]
    WrongTokenType,
}

impl From<AuthError> for ApiError {
    fn from(error: AuthError) -> Self {
        let code = match error {
            AuthError::MissingToken => "missing_token",
            AuthError::InvalidToken => "invalid_token",
            AuthError::ExpiredToken => "expired_token",
            AuthError::WrongTokenType => "wrong_token_type",
        };
        Self::Unauthorized(Reason::new(code, error.to_string()))
    }
}
