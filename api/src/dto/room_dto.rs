//! Wire shapes of the room domain.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::engine::{Layout, Piece};
use crate::models::room_model::{RoomMode, RoomStatus};
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
    /// A finished room, for its players: whether it ended on a surrender rather than a find.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surrendered: Option<bool>,
    /// A composing Duel: the layout the caller already hid, never the opponent's.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Object>)]
    pub hidden_layout: Option<Layout>,
    /// The caller's game: absent until it exists, or for a non-member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_id: Option<Uuid>,
    /// The opponent's shot count: absent until the games exist, or for a non-member.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opponent_shots: Option<i64>,
}

impl From<Seen> for RoomView {
    fn from(seen: Seen) -> Self {
        let Seen {
            room,
            member,
            game_id,
            own_layout,
            opponent_shots,
        } = seen;
        Self {
            code: room.code,
            mode: room.mode,
            status: room.status,
            host: room.host_login,
            opponent: room.opponent_login.filter(|_| member),
            winner: room.winner_login.filter(|_| member),
            surrendered: (member && room.status == RoomStatus::Finished)
                .then_some(room.surrendered),
            game_id,
            hidden_layout: own_layout,
            opponent_shots,
        }
    }
}
