//! The `room` row, with its players' logins.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "room_mode", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RoomMode {
    Race,
    Duel,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "room_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RoomStatus {
    Waiting,
    Composing,
    Playing,
    Finished,
}

#[derive(Debug, Clone)]
pub struct Room {
    pub id: Uuid,
    pub code: String,
    pub mode: RoomMode,
    pub status: RoomStatus,
    pub host_id: Uuid,
    pub host_login: String,
    pub opponent_id: Option<Uuid>,
    pub opponent_login: Option<String>,
    pub winner_login: Option<String>,
    pub surrendered: bool,
}

impl Room {
    pub fn has_player(&self, user_id: Uuid) -> bool {
        self.host_id == user_id || self.opponent_id == Some(user_id)
    }
}

/// The room of a game, locked to play in it or end it, seen from that game's player.
#[derive(Debug, Clone, Copy)]
pub struct RoomLock {
    pub id: Uuid,
    pub opponent_id: Uuid,
    pub mode: RoomMode,
    /// How long ago that player acted, when they acted last; `None` otherwise.
    pub since_own_action_ms: Option<i64>,
    /// Entries that player has fired.
    pub fired: i64,
}

/// Where a Duel's priority stands for one of its players.
#[derive(Debug, Clone)]
pub struct DuelTurn {
    pub last_actor_login: Option<String>,
    pub since_own_action_ms: Option<i64>,
    pub fired: i64,
}

/// The opponent's priority after a player's action: 10 s × the entries that player has fired / 36.
pub const fn priority_ms(fired: i64) -> i64 {
    10_000 * fired / 36
}

/// What is left of the opponent's priority after a player's own action.
pub fn priority_left_ms(fired: i64, since_own_action_ms: Option<i64>) -> u64 {
    since_own_action_ms.map_or(0, |since| {
        u64::try_from(priority_ms(fired) - since).unwrap_or(0)
    })
}
