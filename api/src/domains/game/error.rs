//! Failures of the game domain — no HTTP here but the mapping at the bottom.

use crate::engine::{DrawError, LayoutError, Shape, TraceError};
use crate::shared::error::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum GameError {
    /// Also what another player's game answers: its existence is not the caller's business.
    #[error("Cette partie n'existe pas")]
    NotFound,

    #[error("Cette partie est déjà résolue")]
    AlreadySolved,

    #[error("Cette partie a été abandonnée")]
    Surrendered,

    #[error("Une entrée est numérotée de 1 à 36")]
    InvalidEntry,

    #[error("{}", broken_rule(.0))]
    InvalidLayout(#[from] LayoutError),

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
            GameError::AlreadySolved | GameError::Surrendered => Self::Conflict(message),
            GameError::InvalidEntry | GameError::InvalidLayout(_) => {
                Self::UnprocessableBody(message)
            }
            // Engine failures are bugs, not something the player can act on.
            GameError::Draw(cause) => Self::Internal(cause.into()),
            GameError::Trace(cause) => Self::Internal(cause.into()),
        }
    }
}

/// The engine's layout rule, told to the player.
fn broken_rule(error: &LayoutError) -> String {
    match *error {
        LayoutError::MissingShape(shape) => format!("Il manque la pièce « {} »", name(shape)),
        LayoutError::DuplicateShape(shape) => {
            format!("La pièce « {} » est placée plus d'une fois", name(shape))
        }
        LayoutError::OutsideGrid(shape) => {
            format!("La pièce « {} » dépasse de la grille", name(shape))
        }
        LayoutError::Overlap(a, b) => {
            format!(
                "Les pièces « {} » et « {} » se chevauchent",
                name(a),
                name(b)
            )
        }
        LayoutError::SharedSide(a, b) => format!(
            "Les pièces « {} » et « {} » se touchent par un côté",
            name(a),
            name(b)
        ),
    }
}

const fn name(shape: Shape) -> &'static str {
    match shape {
        Shape::Square => "carré",
        Shape::Bar => "barre",
        Shape::Spike => "pointe",
        Shape::Corner => "coin",
        Shape::Diamond => "losange",
    }
}
