//! Wire shapes of the game domain.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::domains::game::model::{Game, GameStatus, GameSummary, Shot, ShotColour};
use crate::domains::game::service::Verdict;
use crate::engine::{Layout, Piece};

/// A game as its player sees it.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct GameView {
    pub id: Uuid,
    pub status: GameStatus,
    pub created_at: DateTime<Utc>,
    pub solved_at: Option<DateTime<Utc>>,
    /// In firing order.
    pub shots: Vec<ShotResponse>,
    /// The hidden layout: absent from the payload until the game is solved or surrendered.
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

/// Where the beam fired from `entry` left the grid, and the colour it picked up.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
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

/// Whether the proposition answers every entry like the hidden layout.
#[derive(Debug, Clone, Serialize)]
pub struct PropositionResponse {
    pub correct: bool,
    /// The hidden layout, only when the proposition was correct.
    #[serde(skip_serializing_if = "Option::is_none")]
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

/// What a player sends on a game's socket; the first message must be `auth`.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Auth {
        token: String,
    },
    /// Bounds are checked by the service through `Entry::new`.
    Fire {
        entry: u8,
    },
    /// A draft submitted as the answer; the layout rules are the engine's, checked by the service.
    Propose {
        pieces: Vec<Piece>,
    },
    Surrender,
}

/// One answer per client message, in order, plus what another socket on the game changed.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// On connecting, after a surrender, and pushed when the game ends elsewhere.
    Game {
        game: GameView,
    },
    /// `replayed` when the entry had been fired before: the stored shot.
    Shot {
        shot: ShotResponse,
        replayed: bool,
    },
    Verdict(PropositionResponse),
    /// The status and detail an HTTP endpoint would have answered.
    Error {
        action: Action,
        status: u16,
        detail: Value,
    },
}

/// The client message an error answers; `connect` errors close the socket.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Connect,
    Fire,
    Propose,
    Surrender,
    /// The message could not be read.
    Message,
}
