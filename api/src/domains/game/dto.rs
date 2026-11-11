//! Wire shapes of the game domain.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domains::game::model::{Game, GameStatus, GameSummary, Shot, ShotColour};
use crate::domains::game::service::Verdict;
use crate::engine::{Layout, Piece};

/// A game as its player sees it.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GameView {
    pub id: Uuid,
    pub status: GameStatus,
    pub created_at: DateTime<Utc>,
    pub solved_at: Option<DateTime<Utc>>,
    /// In firing order.
    pub shots: Vec<ShotResponse>,
    /// The hidden layout: absent from the payload until the game is solved.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Object>)]
    pub layout: Option<Layout>,
}

impl GameView {
    pub fn new(game: &Game, shots: Vec<Shot>) -> Self {
        Self {
            id: game.id,
            status: game.status,
            created_at: game.created_at,
            solved_at: game.solved_at,
            shots: shots.into_iter().map(Into::into).collect(),
            layout: game.revealed_layout().cloned(),
        }
    }
}

/// One line of the caller's game list — never the layout.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GameSummaryResponse {
    pub id: Uuid,
    pub status: GameStatus,
    pub created_at: DateTime<Utc>,
    pub solved_at: Option<DateTime<Utc>>,
}

impl From<GameSummary> for GameSummaryResponse {
    fn from(summary: GameSummary) -> Self {
        Self {
            id: summary.id,
            status: summary.status,
            created_at: summary.created_at,
            solved_at: summary.solved_at,
        }
    }
}

/// `POST /games/{game_id}/shots`
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct ShotRequest {
    /// Bounds are checked by the service through `Entry::new`.
    #[schema(example = 7, minimum = 1, maximum = 36)]
    pub entry: u8,
}

/// Where the beam fired from `entry` left the grid, and the colour it picked up.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ShotResponse {
    #[schema(example = 7)]
    pub entry: i16,
    #[schema(example = 23)]
    pub exit_entry: i16,
    pub colour: ShotColour,
}

impl From<Shot> for ShotResponse {
    fn from(shot: Shot) -> Self {
        Self {
            entry: shot.entry,
            exit_entry: shot.exit_entry,
            colour: shot.colour,
        }
    }
}

/// `POST /games/{game_id}/propositions` — a draft submitted as the answer.
/// The layout rules are the engine's, checked by the service.
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct PropositionRequest {
    #[schema(value_type = Vec<Object>, example = json!([
        { "shape": "square", "colour": "red", "x": 5, "y": 0, "rotation": 0 },
        { "shape": "bar", "colour": "red", "x": 7, "y": 3, "rotation": 1 },
        { "shape": "spike", "colour": "white", "x": 5, "y": 4, "rotation": 0 },
        { "shape": "corner", "colour": "blue", "x": 3, "y": 2, "rotation": 0 },
        { "shape": "diamond", "colour": "yellow", "x": 1, "y": 5, "rotation": 0 }
    ]))]
    pub pieces: Vec<Piece>,
}

impl PropositionRequest {
    pub fn into_layout(self) -> Layout {
        Layout {
            pieces: self.pieces,
        }
    }
}

/// Whether the proposition answers every entry like the hidden layout.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PropositionResponse {
    pub correct: bool,
    /// The hidden layout, only when the proposition was correct.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Object>)]
    pub layout: Option<Layout>,
}

impl From<Verdict> for PropositionResponse {
    fn from(verdict: Verdict) -> Self {
        match verdict {
            Verdict::Wrong => Self {
                correct: false,
                layout: None,
            },
            Verdict::Correct(game) => Self {
                correct: true,
                layout: game.revealed_layout().cloned(),
            },
        }
    }
}
