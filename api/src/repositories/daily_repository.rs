//! Every SQL statement of the daily domain, and nothing else.

use chrono::NaiveDate;
use sqlx::PgPool;

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
