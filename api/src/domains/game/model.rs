//! The `game` and `shot` rows, and the predicates on one of them.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

use crate::engine::{BeamColour, Layout};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "game_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    InProgress,
    Solved,
    Surrendered,
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

    /// For the engine only: tracing a shot needs the pieces.
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
}

impl Game {
    /// The hidden layout, once the game is over: solved or surrendered.
    pub fn revealed_layout(&self) -> Option<&Layout> {
        (self.status != GameStatus::InProgress).then(|| self.layout.pieces())
    }
}

/// A game without its layout, for listings.
#[derive(Debug, Clone)]
pub struct GameSummary {
    pub id: Uuid,
    pub status: GameStatus,
    pub created_at: DateTime<Utc>,
    pub solved_at: Option<DateTime<Utc>>,
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
