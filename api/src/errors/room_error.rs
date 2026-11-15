//! Failures of the room domain — no HTTP here but the mapping at the bottom.

use crate::engine::{DrawError, LayoutError};
use crate::errors::game_error::broken_rule;
use crate::shared::error::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum RoomError {
    #[error("Cette salle n'existe pas")]
    NotFound,

    /// Guests may only play the daily.
    #[error("Il faut un compte pour jouer à deux : inscrivez-vous pour continuer")]
    AccountRequired,

    #[error("Cette salle est déjà complète")]
    Full,

    #[error("On ne rejoint pas sa propre salle")]
    OwnRoom,

    #[error("Vous ne jouez pas dans cette salle")]
    NotAPlayer,

    /// Only a Duel, once both players are seated and before it starts.
    #[error("Cette salle n'attend pas de disposition")]
    NotComposing,

    #[error("Votre disposition est déjà cachée")]
    LayoutAlreadyHidden,

    #[error("{}", broken_rule(.0))]
    InvalidLayout(#[from] LayoutError),

    #[error(transparent)]
    Draw(#[from] DrawError),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl From<RoomError> for ApiError {
    fn from(error: RoomError) -> Self {
        let message = error.to_string();
        match error {
            RoomError::Database(cause) => Self::Database(cause),
            RoomError::NotFound => Self::NotFound(message),
            RoomError::AccountRequired => Self::Forbidden(message),
            RoomError::Full
            | RoomError::OwnRoom
            | RoomError::NotAPlayer
            | RoomError::NotComposing
            | RoomError::LayoutAlreadyHidden => Self::Conflict(message),
            RoomError::InvalidLayout(_) => Self::UnprocessableBody(message),
            // A bug, not something the player can act on.
            RoomError::Draw(cause) => Self::Internal(cause.into()),
        }
    }
}
