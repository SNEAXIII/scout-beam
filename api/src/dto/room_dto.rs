//! Wire shapes of the room domain.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::room_model::{RoomMode, RoomStatus};
use crate::services::room_service::Seen;

/// Opens a waiting room hosted by the caller.
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateRoomRequest {
    pub mode: RoomMode,
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
}

impl From<Seen> for RoomView {
    fn from(seen: Seen) -> Self {
        let Seen {
            room,
            member,
            game_id,
        } = seen;
        Self {
            code: room.code,
            mode: room.mode,
            status: room.status,
            host: room.host_login,
            opponent: room.opponent_login.filter(|_| member),
            winner: room.winner_login.filter(|_| member),
            game_id,
        }
    }
}
