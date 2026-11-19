//! Every SQL statement of the game domain, and nothing else.

use chrono::NaiveDate;
use sqlx::types::Json;
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::engine::Layout;
use crate::models::game_model::{
    Game, GameStatus, GameSummary, Pace, SealedLayout, Shot, ShotColour,
};

/// The caller's game, or nothing — another player's game is indistinguishable from a missing one.
pub async fn find_owned(pool: &PgPool, game_id: Uuid, user_id: Uuid) -> sqlx::Result<Option<Game>> {
    sqlx::query_as!(
        Game,
        r#"SELECT id, user_id, layout as "layout: SealedLayout", status as "status: GameStatus",
                  created_at, solved_at, surrendered_at, daily_date, room_id,
                  (SELECT code FROM room WHERE room.id = game.room_id) as room_code,
                  (SELECT count(*) FROM proposition
                   WHERE game_id = game.id AND NOT correct) as "wrong_propositions!"
           FROM game
           WHERE id = $1 AND user_id = $2"#,
        game_id,
        user_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_for_user(pool: &PgPool, user_id: Uuid) -> sqlx::Result<Vec<GameSummary>> {
    sqlx::query_as!(
        GameSummary,
        r#"SELECT g.id, g.status as "status: GameStatus", g.created_at, g.solved_at,
                  CASE WHEN g.daily_date IS NOT NULL THEN 'daily'
                       WHEN r.id IS NULL THEN 'bot'
                       ELSE r.mode::text END as "mode!",
                  o.login as "opponent?"
           FROM game g
           LEFT JOIN room r ON r.id = g.room_id
           LEFT JOIN "user" o
             ON o.id = CASE WHEN r.host_id = g.user_id THEN r.opponent_id ELSE r.host_id END
           WHERE g.user_id = $1
           ORDER BY g.created_at DESC, g.id DESC"#,
        user_id
    )
    .fetch_all(pool)
    .await
}

pub async fn insert(pool: &PgPool, user_id: Uuid, layout: &SealedLayout) -> sqlx::Result<Game> {
    sqlx::query_as!(
        Game,
        r#"INSERT INTO game (user_id, layout)
           VALUES ($1, $2)
           RETURNING id, user_id, layout as "layout: SealedLayout", status as "status: GameStatus",
                     created_at, solved_at, surrendered_at, daily_date, NULL::uuid as room_id,
                     NULL::text as room_code,
                     0::int8 as "wrong_propositions!""#,
        user_id,
        layout as _
    )
    .fetch_one(pool)
    .await
}

/// A game on a room's layout; the room's own transaction runs it.
pub async fn insert_in_room<'e, E>(
    executor: E,
    user_id: Uuid,
    room_id: Uuid,
    layout: &SealedLayout,
) -> sqlx::Result<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query!(
        "INSERT INTO game (user_id, layout, room_id) VALUES ($1, $2, $3)",
        user_id,
        layout as _,
        room_id
    )
    .execute(executor)
    .await?;
    Ok(())
}

/// The caller's game of that day, if they started it.
pub async fn find_daily(
    pool: &PgPool,
    user_id: Uuid,
    date: NaiveDate,
) -> sqlx::Result<Option<Game>> {
    sqlx::query_as!(
        Game,
        r#"SELECT id, user_id, layout as "layout: SealedLayout", status as "status: GameStatus",
                  created_at, solved_at, surrendered_at, daily_date, room_id,
                  (SELECT code FROM room WHERE room.id = game.room_id) as room_code,
                  (SELECT count(*) FROM proposition
                   WHERE game_id = game.id AND NOT correct) as "wrong_propositions!"
           FROM game
           WHERE user_id = $1 AND daily_date = $2"#,
        user_id,
        date
    )
    .fetch_optional(pool)
    .await
}

/// A game on that day's layout; `None` when the day has no daily or the caller already has its game.
pub async fn insert_daily(
    pool: &PgPool,
    user_id: Uuid,
    date: NaiveDate,
) -> sqlx::Result<Option<Game>> {
    sqlx::query_as!(
        Game,
        r#"INSERT INTO game (user_id, layout, daily_date)
           SELECT $1, layout, date FROM daily WHERE date = $2
           ON CONFLICT (user_id, daily_date) DO NOTHING
           RETURNING id, user_id, layout as "layout: SealedLayout", status as "status: GameStatus",
                     created_at, solved_at, surrendered_at, daily_date, NULL::uuid as room_id,
                     NULL::text as room_code,
                     0::int8 as "wrong_propositions!""#,
        user_id,
        date
    )
    .fetch_optional(pool)
    .await
}

pub async fn shots_of(pool: &PgPool, game_id: Uuid) -> sqlx::Result<Vec<Shot>> {
    sqlx::query_as!(
        Shot,
        r#"SELECT entry, exit_entry, colour as "colour: ShotColour"
           FROM shot
           WHERE game_id = $1
           ORDER BY created_at, id"#,
        game_id
    )
    .fetch_all(pool)
    .await
}

/// Insert, unless that entry was already fired: `None` means the stored shot stands.
pub async fn insert_shot_if_absent<'e, E>(
    executor: E,
    game_id: Uuid,
    shot: &Shot,
) -> sqlx::Result<Option<Shot>>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as!(
        Shot,
        r#"INSERT INTO shot (game_id, entry, exit_entry, colour)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (game_id, entry) DO NOTHING
           RETURNING entry, exit_entry, colour as "colour: ShotColour""#,
        game_id,
        shot.entry,
        shot.exit_entry,
        shot.colour as _
    )
    .fetch_optional(executor)
    .await
}

pub async fn find_shot(pool: &PgPool, game_id: Uuid, entry: i16) -> sqlx::Result<Option<Shot>> {
    sqlx::query_as!(
        Shot,
        r#"SELECT entry, exit_entry, colour as "colour: ShotColour"
           FROM shot
           WHERE game_id = $1 AND entry = $2"#,
        game_id,
        entry
    )
    .fetch_optional(pool)
    .await
}

/// A room game's pace, its row locked until the caller's transaction ends; `None` outside a room.
pub async fn pace_of<'e, E>(executor: E, game_id: Uuid) -> sqlx::Result<Option<Pace>>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as!(
        Pace,
        r#"SELECT (extract(epoch from now() - created_at) * 1000)::int8 as "elapsed_ms!",
                  (SELECT count(*) FROM shot WHERE game_id = game.id) as "fired!",
                  (SELECT count(*) FROM proposition
                   WHERE game_id = game.id AND NOT correct) as "wrong_propositions!",
                  (SELECT (extract(epoch from now() - max(created_at)) * 1000)::int8
                   FROM proposition WHERE game_id = game.id) as since_proposition_ms
           FROM game
           WHERE id = $1 AND room_id IS NOT NULL
           FOR UPDATE"#,
        game_id
    )
    .fetch_optional(executor)
    .await
}

/// The game's status, its row locked until the caller's transaction ends.
pub async fn lock_status<'e, E>(executor: E, game_id: Uuid) -> sqlx::Result<Option<GameStatus>>
where
    E: PgExecutor<'e>,
{
    sqlx::query_scalar!(
        r#"SELECT status as "status: GameStatus" FROM game WHERE id = $1 FOR UPDATE"#,
        game_id
    )
    .fetch_optional(executor)
    .await
}

pub async fn insert_proposition<'e, E>(
    executor: E,
    game_id: Uuid,
    layout: &Layout,
    correct: bool,
) -> sqlx::Result<()>
where
    E: PgExecutor<'e>,
{
    sqlx::query!(
        "INSERT INTO proposition (game_id, layout, correct) VALUES ($1, $2, $3)",
        game_id,
        Json(layout) as _,
        correct
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn mark_surrendered<'e, E>(executor: E, game_id: Uuid) -> sqlx::Result<Game>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as!(
        Game,
        r#"UPDATE game SET status = 'surrendered', surrendered_at = now()
           WHERE id = $1
           RETURNING id, user_id, layout as "layout: SealedLayout", status as "status: GameStatus",
                     created_at, solved_at, surrendered_at, daily_date, room_id,
                     (SELECT code FROM room WHERE room.id = game.room_id) as room_code,
                     (SELECT count(*) FROM proposition
                      WHERE game_id = game.id AND NOT correct) as "wrong_propositions!""#,
        game_id
    )
    .fetch_one(executor)
    .await
}

pub async fn mark_solved<'e, E>(executor: E, game_id: Uuid) -> sqlx::Result<Game>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as!(
        Game,
        r#"UPDATE game SET status = 'solved', solved_at = now()
           WHERE id = $1
           RETURNING id, user_id, layout as "layout: SealedLayout", status as "status: GameStatus",
                     created_at, solved_at, surrendered_at, daily_date, room_id,
                     (SELECT code FROM room WHERE room.id = game.room_id) as room_code,
                     (SELECT count(*) FROM proposition
                      WHERE game_id = game.id AND NOT correct) as "wrong_propositions!""#,
        game_id
    )
    .fetch_one(executor)
    .await
}
