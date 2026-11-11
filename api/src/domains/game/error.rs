//! Failures of the game domain — no HTTP here but the mapping at the bottom.

use crate::engine::{DrawError, TraceError};
use crate::shared::error::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum GameError {
    /// Also what another player's game answers: its existence is not the caller's business.
    #[error("Cette partie n'existe pas")]
    NotFound,

    #[error("Cette partie est déjà résolue")]
    AlreadySolved,

    #[error("Une entrée est numérotée de 1 à 36")]
    InvalidEntry,

    #[error(transparent)]
    Draw(#[from] DrawError),

    #[error(transparent)]
    Trace(#[from] TraceError),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl From<GameError> for ApiError {
    fn from(error: GameError) -> Self {
        let message = error.to_string();
        match error {
            GameError::Database(cause) => Self::Database(cause),
            GameError::NotFound => Self::NotFound(message),
            GameError::AlreadySolved => Self::Conflict(message),
            GameError::InvalidEntry => Self::UnprocessableBody(message),
            // Engine failures are bugs, not something the player can act on.
            GameError::Draw(cause) => Self::Internal(cause.into()),
            GameError::Trace(cause) => Self::Internal(cause.into()),
        }
    }
}
