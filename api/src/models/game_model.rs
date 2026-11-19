//! The `game` and `shot` rows, and the predicates on one of them.

use std::fmt;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

use crate::engine::{BeamColour, Layout, pace};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "game_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    InProgress,
    Solved,
    Surrendered,
    /// A room game whose opponent won.
    Lost,
}

/// The hidden layout as stored. No `Serialize`, and a `Debug` that prints nothing: it cannot
/// reach a response or a log line by accident, only through `Game::revealed_layout`.
#[derive(Clone, sqlx::Type)]
#[sqlx(transparent)]
pub struct SealedLayout(Json<Layout>);

impl SealedLayout {
    pub const fn new(layout: Layout) -> Self {
        Self(Json(layout))
    }

    /// For the engine, which traces shots with it, and for the player who hid it in a Duel.
    pub fn pieces(&self) -> &Layout {
        &self.0
    }
}

impl fmt::Debug for SealedLayout {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SealedLayout(..)")
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    pub id: Uuid,
    pub user_id: Uuid,
    pub layout: SealedLayout,
    pub status: GameStatus,
    pub created_at: DateTime<Utc>,
    pub solved_at: Option<DateTime<Utc>>,
    pub surrendered_at: Option<DateTime<Utc>>,
    /// The day it was the daily of; `None` for a game against the bot.
    pub daily_date: Option<NaiveDate>,
    pub room_id: Option<Uuid>,
    /// The room it is played in; `None` outside a room.
    pub room_code: Option<String>,
    pub wrong_propositions: i64,
}

impl Game {
    /// The hidden layout, once the game is over: solved, surrendered or lost.
    pub fn revealed_layout(&self) -> Option<&Layout> {
        (self.status != GameStatus::InProgress).then(|| self.layout.pieces())
    }
}

/// Where a room game's shot allowance and proposition cooldown stand, at the database's `now()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pace {
    pub elapsed_ms: i64,
    /// New shots fired so far.
    pub fired: i64,
    pub wrong_propositions: i64,
    /// Since the last proposition; `None` before the first.
    pub since_proposition_ms: Option<i64>,
}

impl Pace {
    pub const fn shots_left(&self) -> i64 {
        pace::shots_left(self.elapsed_ms, self.fired)
    }

    pub fn next_shot_in_ms(&self) -> u64 {
        pace::next_shot_in_ms(self.elapsed_ms)
    }

    pub fn proposition_in_ms(&self) -> u64 {
        pace::proposition_in_ms(self.wrong_propositions, self.since_proposition_ms)
    }
}

/// A game without its layout, for listings.
#[derive(Debug, Clone)]
pub struct GameSummary {
    pub id: Uuid,
    pub status: GameStatus,
    pub created_at: DateTime<Utc>,
    pub solved_at: Option<DateTime<Utc>>,
    /// When it was solved or surrendered; `None` while in progress.
    pub ended_at: Option<DateTime<Utc>>,
    /// `bot`, `daily`, `race` or `duel`.
    pub mode: String,
    pub opponent: Option<String>,
    /// A finished Bot game's seconds to its end, penalties left out; `None` for any other game.
    pub seconds: Option<i64>,
    /// A finished Bot game's score; `None` for any other game.
    pub score: Option<i64>,
    pub points: Option<i64>,
}

/// The Bot games of one player, finished ones with their score and points, oldest first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BotGameStats {
    pub played: i64,
    pub solved: i64,
    pub surrendered: i64,
    /// Fewest raw seconds of a solved Bot game, penalties not added.
    pub best_seconds: Option<i64>,
    pub average_points: Option<i64>,
    pub recent: Vec<BotGameRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BotGameRecord {
    pub ended_at: DateTime<Utc>,
    pub score: i64,
    pub points: i64,
    pub surrendered: bool,
}

/// The engine's beam colour as the `beam_colour` column stores it.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "beam_colour", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ShotColour {
    None,
    White,
    Red,
    Blue,
    Yellow,
    Violet,
    Orange,
    Green,
    Black,
}

impl From<BeamColour> for ShotColour {
    fn from(colour: BeamColour) -> Self {
        match colour {
            BeamColour::None => Self::None,
            BeamColour::White => Self::White,
            BeamColour::Red => Self::Red,
            BeamColour::Blue => Self::Blue,
            BeamColour::Yellow => Self::Yellow,
            BeamColour::Violet => Self::Violet,
            BeamColour::Orange => Self::Orange,
            BeamColour::Green => Self::Green,
            BeamColour::Black => Self::Black,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Shot {
    pub entry: i16,
    pub exit_entry: i16,
    pub colour: ShotColour,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn a_sealed_layout_prints_nothing_of_its_pieces() {
        let mut rng = rand::rng();
        let layout = crate::engine::draw_layout(&mut rng).unwrap();
        let game = Game {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            layout: SealedLayout::new(layout),
            status: GameStatus::InProgress,
            created_at: Utc::now(),
            solved_at: None,
            surrendered_at: None,
            daily_date: None,
            room_id: None,
            room_code: None,
            wrong_propositions: 0,
        };

        let printed = format!("{game:?}");

        assert!(printed.contains("SealedLayout(..)"));
        assert!(!printed.contains("pieces"));
        assert!(game.revealed_layout().is_none());
    }

    #[test]
    fn every_beam_colour_keeps_its_name_in_storage() {
        for colour in [
            BeamColour::None,
            BeamColour::White,
            BeamColour::Red,
            BeamColour::Blue,
            BeamColour::Yellow,
            BeamColour::Violet,
            BeamColour::Orange,
            BeamColour::Green,
            BeamColour::Black,
        ] {
            assert_eq!(
                serde_json::to_value(ShotColour::from(colour)).unwrap(),
                serde_json::to_value(colour).unwrap()
            );
        }
    }
}
