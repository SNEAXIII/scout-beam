//! Wire shapes of the room domain.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::engine::Piece;
use crate::models::room_model::{RoomMode, RoomStatus, priority_left_ms};
use crate::services::room_service::Seen;

/// Opens a waiting room hosted by the caller.
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateRoomRequest {
    pub mode: RoomMode,
}

/// The layout the caller hides for the opponent; the rules are the engine's, checked by the service.
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct HideLayoutRequest {
    #[schema(value_type = Vec<Object>)]
    pub pieces: Vec<Piece>,
}

/// A room as one of its players sees it; anyone else gets only the code, mode, status and host.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RoomView {
    /// Also the invitation link: `/rooms/<code>`.
    #[schema(example = "K7RX2M")]
    pub code: String,
    pub mode: RoomMode,
    pub status: RoomStatus,
    /// The host's login.
    pub host: String,
    /// The opponent's login: absent until someone joins, or for a non-member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opponent: Option<String>,
    /// The winner's login: absent until the room ends, or for a non-member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner: Option<String>,
    /// The caller's game: absent until it exists, or for a non-member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_id: Option<Uuid>,
    /// A playing Duel only: the login of whoever acted last, absent before any action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_actor: Option<String>,
    /// A playing Duel only: milliseconds before the caller may act again, 0 when they may now.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<u64>,
}

impl From<Seen> for RoomView {
    fn from(seen: Seen) -> Self {
        let Seen {
            room,
            member,
            game_id,
            turn,
        } = seen;
        Self {
            code: room.code,
            mode: room.mode,
            status: room.status,
            host: room.host_login,
            opponent: room.opponent_login.filter(|_| member),
            winner: room.winner_login.filter(|_| member),
            game_id,
            retry_after_ms: turn
                .as_ref()
                .map(|turn| priority_left_ms(turn.fired, turn.since_own_action_ms)),
            last_actor: turn.and_then(|turn| turn.last_actor_login),
        }
    }
}
