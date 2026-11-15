//! Every SQL statement of the daily domain, and nothing else.

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::daily_model::{DayRankingRow, GlobalRankingRow};
use crate::models::game_model::SealedLayout;

/// The day changes at midnight in Paris, whatever the server's time zone.
pub async fn today(pool: &PgPool) -> sqlx::Result<NaiveDate> {
    sqlx::query_scalar!(r#"SELECT (now() AT TIME ZONE 'Europe/Paris')::date as "today!""#)
        .fetch_one(pool)
        .await
}

/// Store the day's layout unless one is already there: the first stored stands.
pub async fn insert_if_absent(
    pool: &PgPool,
    date: NaiveDate,
    layout: &SealedLayout,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO daily (date, layout) VALUES ($1, $2) ON CONFLICT (date) DO NOTHING",
        date,
        layout as _
    )
    .execute(pool)
    .await?;
    Ok(())
}

// ponytail: score = seconds + 60 per wrong proposition, points = max(100, 1000 - score / 6);
// weights to refine after real games, e.g. a cost per shot.

/// That day's solved dailies, started on that same Paris date, by registered players: the top 100
/// and the caller's row wherever it falls.
pub async fn day_ranking(
    pool: &PgPool,
    date: NaiveDate,
    caller: Option<Uuid>,
) -> sqlx::Result<Vec<DayRankingRow>> {
    sqlx::query_as!(
        DayRankingRow,
        r#"WITH timed AS (
               SELECT u.id AS user_id, u.login, g.solved_at,
                      floor(EXTRACT(EPOCH FROM g.solved_at - g.created_at))::int8 AS seconds,
                      (SELECT count(*) FROM proposition p
                       WHERE p.game_id = g.id AND NOT p.correct) AS wrong
               FROM game g
               JOIN "user" u ON u.id = g.user_id
               WHERE g.daily_date = $1 AND g.status = 'solved' AND NOT u.guest
                 AND (g.created_at AT TIME ZONE 'Europe/Paris')::date = $1
           ), ranked AS (
               SELECT *, RANK() OVER (ORDER BY seconds + 60 * wrong) AS rank,
                      ROW_NUMBER() OVER (ORDER BY seconds + 60 * wrong, solved_at, login) AS position
               FROM timed
           )
           SELECT rank as "rank!", login as "login!", seconds + 60 * wrong as "score!",
                  seconds as "seconds!", wrong as "wrong_propositions!",
                  position <= 100 as "in_top!", user_id IS NOT DISTINCT FROM $2 as "mine!"
           FROM ranked
           WHERE position <= 100 OR user_id IS NOT DISTINCT FROM $2
           ORDER BY position"#,
        date,
        caller
    )
    .fetch_all(pool)
    .await
}

/// Points over every solved daily, late ones included, by registered players: the top 100 and
/// the caller's row wherever it falls.
pub async fn global_ranking(
    pool: &PgPool,
    caller: Option<Uuid>,
) -> sqlx::Result<Vec<GlobalRankingRow>> {
    sqlx::query_as!(
        GlobalRankingRow,
        r#"WITH scored AS (
               SELECT u.id AS user_id, u.login,
                      floor(EXTRACT(EPOCH FROM g.solved_at - g.created_at))::int8
                      + 60 * (SELECT count(*) FROM proposition p
                              WHERE p.game_id = g.id AND NOT p.correct) AS score
               FROM game g
               JOIN "user" u ON u.id = g.user_id
               WHERE g.daily_date IS NOT NULL AND g.status = 'solved' AND NOT u.guest
           ), summed AS (
               SELECT user_id, login, sum(GREATEST(100, 1000 - score / 6))::int8 AS points,
                      count(*) AS dailies_solved
               FROM scored
               GROUP BY user_id, login
           ), ranked AS (
               SELECT *, RANK() OVER (ORDER BY points DESC) AS rank,
                      ROW_NUMBER() OVER (ORDER BY points DESC, login) AS position
               FROM summed
           )
           SELECT rank as "rank!", login as "login!", points as "points!",
                  dailies_solved as "dailies_solved!",
                  position <= 100 as "in_top!", user_id IS NOT DISTINCT FROM $1 as "mine!"
           FROM ranked
           WHERE position <= 100 OR user_id IS NOT DISTINCT FROM $1
           ORDER BY position"#,
        caller
    )
    .fetch_all(pool)
    .await
}
