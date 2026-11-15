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

/// The room of a game, locked to end it, seen from that game's player.
#[derive(Debug, Clone, Copy)]
pub struct RoomLock {
    pub id: Uuid,
    pub opponent_id: Uuid,
}
