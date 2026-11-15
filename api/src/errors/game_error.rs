//! Failures of the game domain — no HTTP here but the mapping at the bottom.

use serde_json::{Value, json};

use crate::engine::{DrawError, LayoutError, TraceError};
use crate::shared::error::{ApiError, Reason};

#[derive(Debug, thiserror::Error)]
pub enum GameError {
    /// Also what another player's game answers: its existence is not the caller's business.
    #[error("This game does not exist")]
    NotFound,

    /// Guests may only play the daily.
    #[error("Starting a game takes an account: sign up to go on")]
    AccountRequired,

    #[error("There is no daily on that day")]
    DailyNotFound,

    #[error("A daily cannot be surrendered")]
    DailySurrender,

    #[error("This game is already solved")]
    AlreadySolved,

    #[error("This game was surrendered")]
    Surrendered,

    #[error("This game is lost: the opponent won")]
    Lost,

    /// A room game's shot allowance is spent until the next earned shot.
    #[error("No shot left: the next one in {} s", .retry_after_ms.div_ceil(1000))]
    NoShotLeft { retry_after_ms: u64 },

    #[error("Next proposition in {} s", .retry_after_ms.div_ceil(1000))]
    PropositionCooldown { retry_after_ms: u64 },

    #[error("An entry is numbered from 1 to 36")]
    InvalidEntry,

    #[error("{0}")]
    InvalidLayout(#[from] LayoutError),

    #[error(transparent)]
    Draw(#[from] DrawError),

    #[error(transparent)]
    Trace(#[from] TraceError),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl GameError {
    /// The code the front translates, and what fills its text.
    fn code(&self) -> (&'static str, Value) {
        let code = match self {
            Self::NotFound => "game_not_found",
            Self::AccountRequired => "game_account_required",
            Self::DailyNotFound => "daily_not_found",
            Self::DailySurrender => "daily_surrender",
            Self::AlreadySolved => "already_solved",
            Self::Surrendered => "surrendered",
            Self::Lost => "lost",
            Self::NoShotLeft { retry_after_ms } => {
                return ("no_shot_left", seconds(*retry_after_ms));
            }
            Self::PropositionCooldown { retry_after_ms } => {
                return ("proposition_cooldown", seconds(*retry_after_ms));
            }
            Self::InvalidEntry => "invalid_entry",
            Self::InvalidLayout(error) => return broken_rule(error),
            Self::Draw(_) | Self::Trace(_) | Self::Database(_) => "internal",
        };
        (code, Value::Null)
    }
}

fn seconds(retry_after_ms: u64) -> Value {
    json!({ "seconds": retry_after_ms.div_ceil(1000) })
}

impl From<GameError> for ApiError {
    fn from(error: GameError) -> Self {
        let (code, params) = error.code();
        let reason = Reason {
            code,
            params,
            message: error.to_string(),
        };
        match error {
            GameError::Database(cause) => Self::Database(cause),
            GameError::NotFound | GameError::DailyNotFound => Self::NotFound(reason),
            GameError::AccountRequired => Self::Forbidden(reason),
            GameError::AlreadySolved
            | GameError::Surrendered
            | GameError::Lost
            | GameError::DailySurrender => Self::Conflict(reason),
            GameError::NoShotLeft { retry_after_ms }
            | GameError::PropositionCooldown { retry_after_ms } => Self::RetryLater {
                reason,
                retry_after_ms,
            },
            GameError::InvalidEntry | GameError::InvalidLayout(_) => {
                Self::UnprocessableBody(reason)
            }
            // Engine failures are bugs, not something the player can act on.
            GameError::Draw(cause) => Self::Internal(cause.into()),
            GameError::Trace(cause) => Self::Internal(cause.into()),
        }
    }
}

/// The engine's layout rule, as a code and the shapes or colour it names.
pub fn broken_rule(error: &LayoutError) -> (&'static str, Value) {
    match *error {
        LayoutError::MissingShape(shape) => ("missing_shape", json!({ "shape": shape })),
        LayoutError::DuplicateShape(shape) => ("duplicate_shape", json!({ "shape": shape })),
        LayoutError::OutsideGrid(shape) => ("shape_outside_grid", json!({ "shape": shape })),
        LayoutError::Overlap(shape, other) => {
            ("shapes_overlap", json!({ "shape": shape, "other": other }))
        }
        LayoutError::SharedSide(shape, other) => (
            "shapes_share_side",
            json!({ "shape": shape, "other": other }),
        ),
        LayoutError::MissingColour(colour) => ("missing_colour", json!({ "colour": colour })),
    }
}
