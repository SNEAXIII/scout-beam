//! Every SQL statement of the room domain, and nothing else.

use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::models::game_model::SealedLayout;
use crate::models::room_model::{DuelTurn, Room, RoomLock, RoomMode, RoomStatus};

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
                       ELSE room.host_id END as "opponent_id!",
                  room.mode as "mode: RoomMode",
                  CASE WHEN room.last_actor_id = game.user_id
                       THEN (extract(epoch from now() - room.last_action_at) * 1000)::int8
                  END as since_own_action_ms,
                  (SELECT count(*) FROM shot WHERE shot.game_id = game.id) as "fired!"
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

/// The Duel priority as the player of that game sees it.
pub async fn turn_of(pool: &PgPool, game_id: Uuid) -> sqlx::Result<DuelTurn> {
    sqlx::query_as!(
        DuelTurn,
        r#"SELECT last_actor.login as "last_actor_login?",
                  CASE WHEN room.last_actor_id = game.user_id
                       THEN (extract(epoch from now() - room.last_action_at) * 1000)::int8
                  END as since_own_action_ms,
                  (SELECT count(*) FROM shot WHERE shot.game_id = game.id) as "fired!"
           FROM game
           JOIN room ON room.id = game.room_id
           LEFT JOIN "user" last_actor ON last_actor.id = room.last_actor_id
           WHERE game.id = $1"#,
        game_id
    )
    .fetch_one(pool)
    .await
}

/// The player acted: the opponent has priority from now, the instant answered.
pub async fn record_action<'e, E>(
    executor: E,
    room_id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<DateTime<Utc>>
where
    E: PgExecutor<'e>,
{
    sqlx::query_scalar!(
        r#"UPDATE room SET last_actor_id = $2, last_action_at = now() WHERE id = $1
           RETURNING last_action_at as "last_action_at!""#,
        room_id,
        user_id
    )
    .fetch_one(executor)
    .await
}

/// Store the player's layout in a composing room: both layouts once stored, `None` when theirs
/// already was. The row lock makes exactly one of two racing calls see both.
pub async fn hide_layout<'e, E>(
    executor: E,
    room_id: Uuid,
    user_id: Uuid,
    layout: &SealedLayout,
) -> sqlx::Result<Option<(Option<SealedLayout>, Option<SealedLayout>)>>
where
    E: PgExecutor<'e>,
{
    let hidden = sqlx::query!(
        r#"UPDATE room
           SET host_layout = CASE WHEN host_id = $2 THEN $3 ELSE host_layout END,
               opponent_layout = CASE WHEN opponent_id = $2 THEN $3 ELSE opponent_layout END
           WHERE id = $1 AND status = 'composing'
             AND CASE WHEN host_id = $2 THEN host_layout ELSE opponent_layout END IS NULL
           RETURNING host_layout as "host_layout: SealedLayout",
                     opponent_layout as "opponent_layout: SealedLayout""#,
        room_id,
        user_id,
        layout as _
    )
    .fetch_optional(executor)
    .await?;
    Ok(hidden.map(|row| (row.host_layout, row.opponent_layout)))
}

pub async fn start<'e, E>(executor: E, room_id: Uuid) -> sqlx::Result<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query!("UPDATE room SET status = 'playing' WHERE id = $1", room_id)
        .execute(executor)
        .await?;
    Ok(())
}
