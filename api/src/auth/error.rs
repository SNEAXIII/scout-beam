//! Authentication failures, in authentication vocabulary, phrased in French for the front.

use crate::shared::error::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Aucun token n'a été fourni")]
    MissingToken,

    #[error("Le token fourni est invalide")]
    InvalidToken,

    #[error("Le token fourni a expiré")]
    ExpiredToken,

    #[error("Le token ne contient pas d'utilisateur")]
    MissingSubject,

    #[error("Le rôle du token est invalide")]
    InvalidRole,

    /// A refresh token was presented where an access token is required, or the other way round.
    #[error("Le type de token est invalide")]
    WrongTokenType,
}

impl From<AuthError> for ApiError {
    fn from(error: AuthError) -> Self {
        Self::Unauthorized(error.to_string())
    }
}
