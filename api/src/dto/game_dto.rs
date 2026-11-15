//! Wire shapes of the game domain.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::engine::{Layout, Piece};
use crate::models::game_model::{Game, GameStatus, GameSummary, Pace, Shot, ShotColour};
use crate::services::game_service::Verdict;

/// A game as its player sees it.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct GameView {
    pub id: Uuid,
    pub status: GameStatus,
    pub created_at: DateTime<Utc>,
    pub solved_at: Option<DateTime<Utc>>,
    /// The day it is the daily of; null for a game against the bot.
    #[schema(example = "2026-11-15")]
    pub daily_date: Option<NaiveDate>,
    /// The room it is played in; null outside a room.
    #[schema(example = "K7RX2M")]
    pub room_code: Option<String>,
    /// Propositions refused as wrong; each one adds a minute to a daily's score.
    pub wrong_propositions: i64,
    /// In firing order.
    pub shots: Vec<ShotResponse>,
    /// The hidden layout: absent from the payload until the game is over.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Object>)]
    pub layout: Option<Layout>,
    /// A room game only: absent from the payload outside a room.
    #[serde(flatten)]
    pub pace: Option<PaceView>,
}

/// A room game's pace, relative to the server's clock: the shots left follow from `elapsed_ms`
/// and the shot count.
#[derive(Debug, Clone, Copy, Serialize, utoipa::ToSchema)]
pub struct PaceView {
    /// Since the game started.
    pub elapsed_ms: u64,
    /// Before the next proposition is taken; 0 when it is now.
    pub proposition_in_ms: u64,
}

impl From<Pace> for PaceView {
    fn from(pace: Pace) -> Self {
        Self {
            elapsed_ms: u64::try_from(pace.elapsed_ms).unwrap_or(0),
            proposition_in_ms: pace.proposition_in_ms(),
        }
    }
}

impl GameView {
    pub fn new(game: &Game, shots: Vec<Shot>) -> Self {
        Self {
            id: game.id,
            status: game.status,
            created_at: game.created_at,
            solved_at: game.solved_at,
            daily_date: game.daily_date,
            room_code: game.room_code.clone(),
            wrong_propositions: game.wrong_propositions,
            shots: shots.into_iter().map(Into::into).collect(),
            layout: game.revealed_layout().cloned(),
            pace: None,
        }
    }

    #[must_use]
    pub fn with_pace(self, pace: Option<Pace>) -> Self {
        Self {
            pace: pace.map(Into::into),
            ..self
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
    #[schema(example = "race")]
    pub mode: String,
    /// The other player's login, in a room only.
    pub opponent: Option<String>,
}

impl From<GameSummary> for GameSummaryResponse {
    fn from(summary: GameSummary) -> Self {
        Self {
            id: summary.id,
            status: summary.status,
            created_at: summary.created_at,
            solved_at: summary.solved_at,
            mode: summary.mode,
            opponent: summary.opponent,
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
    /// A room game only, this proposition counted.
    #[serde(flatten)]
    pub pace: Option<PaceView>,
}

impl PropositionResponse {
    pub fn new(verdict: &Verdict, pace: Option<Pace>) -> Self {
        let layout = match verdict {
            Verdict::Wrong => None,
            Verdict::Correct(game) => game.revealed_layout().cloned(),
        };
        Self {
            correct: matches!(verdict, Verdict::Correct(_)),
            layout,
            pace: pace.map(Into::into),
        }
    }
}

/// What a player sends on a game's socket, once the upgrade has authenticated it.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketClientMessage {
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

/// One answer per client message, in order, plus what another socket on the game or its room
/// changed.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketServerMessage {
    /// On connecting, after a surrender, and pushed when the game ends elsewhere.
    Game {
        game: GameView,
    },
    /// `replayed` when the entry had been fired before: the stored shot. In a room, the pace.
    Shot {
        shot: ShotResponse,
        replayed: bool,
        #[serde(flatten)]
        pace: Option<PaceView>,
    },
    Verdict(PropositionResponse),
    /// A room only: the opponent's shot count, after each of their new shots.
    Opponent {
        shots: i64,
    },
    /// The room socket only: the room changed, to be read again.
    Room,
    /// The status and detail an HTTP endpoint would have answered.
    Error {
        action: Action,
        status: u16,
        detail: Value,
        /// Only on a room's shot or proposition refused for its pace: when to try again.
        #[serde(skip_serializing_if = "Option::is_none")]
        retry_after_ms: Option<u64>,
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
