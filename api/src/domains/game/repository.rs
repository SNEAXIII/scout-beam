//! Every SQL statement of the game domain, and nothing else.

use sqlx::types::Json;
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use crate::domains::game::model::{Game, GameStatus, GameSummary, SealedLayout, Shot, ShotColour};
use crate::engine::Layout;

/// The caller's game, or nothing — another player's game is indistinguishable from a missing one.
pub async fn find_owned(pool: &PgPool, game_id: Uuid, user_id: Uuid) -> sqlx::Result<Option<Game>> {
    sqlx::query_as!(
        Game,
        r#"SELECT id, user_id, layout as "layout: SealedLayout", status as "status: GameStatus",
                  created_at, solved_at
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
        r#"SELECT id, status as "status: GameStatus", created_at, solved_at
           FROM game
           WHERE user_id = $1
           ORDER BY created_at DESC, id DESC"#,
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
                     created_at, solved_at"#,
        user_id,
        layout as _
    )
    .fetch_one(pool)
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
pub async fn insert_shot_if_absent(
    pool: &PgPool,
    game_id: Uuid,
    shot: &Shot,
) -> sqlx::Result<Option<Shot>> {
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
    .fetch_optional(pool)
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

pub async fn mark_solved<'e, E>(executor: E, game_id: Uuid) -> sqlx::Result<Game>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as!(
        Game,
        r#"UPDATE game SET status = 'solved', solved_at = now()
           WHERE id = $1
           RETURNING id, user_id, layout as "layout: SealedLayout", status as "status: GameStatus",
                     created_at, solved_at"#,
        game_id
    )
    .fetch_one(executor)
    .await
}
