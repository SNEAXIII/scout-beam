//! Failures of the room domain — no HTTP here but the mapping at the bottom.

use serde_json::Value;

use crate::engine::{DrawError, LayoutError};
use crate::errors::game_error::broken_rule;
use crate::shared::error::{ApiError, Reason};

#[derive(Debug, thiserror::Error)]
pub enum RoomError {
    #[error("This room does not exist")]
    NotFound,

    /// Guests may only play the daily.
    #[error("Playing against someone takes an account: sign up to go on")]
    AccountRequired,

    #[error("This room is already full")]
    Full,

    #[error("A room cannot be joined by its own host")]
    OwnRoom,

    #[error("You are not playing in this room")]
    NotAPlayer,

    /// Only a Duel, once both players are seated and before it starts.
    #[error("This room is not waiting for a layout")]
    NotComposing,

    #[error("Your layout is already hidden")]
    LayoutAlreadyHidden,

    #[error("{0}")]
    InvalidLayout(#[from] LayoutError),

    #[error(transparent)]
    Draw(#[from] DrawError),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl RoomError {
    /// The code the front translates, and what fills its text.
    fn code(&self) -> (&'static str, Value) {
        let code = match self {
            Self::NotFound => "room_not_found",
            Self::AccountRequired => "room_account_required",
            Self::Full => "room_full",
            Self::OwnRoom => "own_room",
            Self::NotAPlayer => "not_a_player",
            Self::NotComposing => "not_composing",
            Self::LayoutAlreadyHidden => "layout_already_hidden",
            Self::InvalidLayout(error) => return broken_rule(error),
            Self::Draw(_) | Self::Database(_) => "internal",
        };
        (code, Value::Null)
    }
}

impl From<RoomError> for ApiError {
    fn from(error: RoomError) -> Self {
        let (code, params) = error.code();
        let reason = Reason {
            code,
            params,
            message: error.to_string(),
        };
        match error {
            RoomError::Database(cause) => Self::Database(cause),
            RoomError::NotFound => Self::NotFound(reason),
            RoomError::AccountRequired => Self::Forbidden(reason),
            RoomError::Full
            | RoomError::OwnRoom
            | RoomError::NotAPlayer
            | RoomError::NotComposing
            | RoomError::LayoutAlreadyHidden => Self::Conflict(reason),
            RoomError::InvalidLayout(_) => Self::UnprocessableBody(reason),
            // A bug, not something the player can act on.
            RoomError::Draw(cause) => Self::Internal(cause.into()),
        }
    }
}
