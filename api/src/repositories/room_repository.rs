//! Every SQL statement of the room domain, and nothing else.

use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::models::room_model::{Room, RoomLock, RoomMode, RoomStatus};

pub async fn find_by_code(pool: &PgPool, code: &str) -> sqlx::Result<Option<Room>> {
    sqlx::query_as!(
        Room,
        r#"SELECT room.id, room.code, room.mode as "mode: RoomMode",
                  room.status as "status: RoomStatus", room.host_id, host.login as "host_login!",
                  room.opponent_id, opponent.login as "opponent_login?",
                  winner.login as "winner_login?"
           FROM room
           JOIN "user" host ON host.id = room.host_id
           LEFT JOIN "user" opponent ON opponent.id = room.opponent_id
           LEFT JOIN "user" winner ON winner.id = room.winner_id
           WHERE room.code = $1"#,
        code
    )
    .fetch_optional(pool)
    .await
}

/// A unique violation means the code is taken: draw another.
pub async fn insert(pool: &PgPool, code: &str, mode: RoomMode, host_id: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO room (code, mode, host_id) VALUES ($1, $2, $3)",
        code,
        mode as _,
        host_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Fill the free seat; `None` when the room is full or the caller hosts it.
pub async fn seat_opponent<'e, E>(
    executor: E,
    code: &str,
    opponent_id: Uuid,
    status: RoomStatus,
) -> sqlx::Result<Option<Uuid>>
where
    E: PgExecutor<'e>,
{
    sqlx::query_scalar!(
        "UPDATE room SET opponent_id = $2, status = $3
         WHERE code = $1 AND opponent_id IS NULL AND host_id <> $2
         RETURNING id",
        code,
        opponent_id,
        status as _
    )
    .fetch_optional(executor)
    .await
}

/// The player's game in that room, once it exists.
pub async fn game_of(pool: &PgPool, room_id: Uuid, user_id: Uuid) -> sqlx::Result<Option<Uuid>> {
    sqlx::query_scalar!(
        "SELECT id FROM game WHERE room_id = $1 AND user_id = $2",
        room_id,
        user_id
    )
    .fetch_optional(pool)
    .await
}

/// The room of that game, locked until the caller's transaction ends; `None` outside a room.
pub async fn lock_for_game<'e, E>(executor: E, game_id: Uuid) -> sqlx::Result<Option<RoomLock>>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as!(
        RoomLock,
        r#"SELECT room.id,
                  CASE WHEN room.host_id = game.user_id THEN room.opponent_id
                       ELSE room.host_id END as "opponent_id!"
           FROM room
           JOIN game ON game.room_id = room.id
           WHERE game.id = $1
           FOR UPDATE OF room"#,
        game_id
    )
    .fetch_optional(executor)
    .await
}

/// The room ends: the winner's game still in progress is solved, the other one lost.
pub async fn finish<'e, E>(executor: E, room_id: Uuid, winner_id: Uuid) -> sqlx::Result<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query!(
        r#"WITH finished AS (
               UPDATE room SET status = 'finished', winner_id = $2 WHERE id = $1
           )
           UPDATE game
           SET status = CASE WHEN user_id = $2 THEN 'solved' ELSE 'lost' END::game_status,
               solved_at = CASE WHEN user_id = $2 THEN now() END
           WHERE room_id = $1 AND status = 'in_progress'"#,
        room_id,
        winner_id
    )
    .execute(executor)
    .await?;
    Ok(())
}
