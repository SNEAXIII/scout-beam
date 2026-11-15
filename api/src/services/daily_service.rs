//! Rules of the daily: one layout a day for every player, one game each.

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::engine;
use crate::errors::game_error::GameError;
use crate::models::game_model::{Game, SealedLayout};
use crate::repositories::{daily_repository, game_repository};

pub async fn today(pool: &PgPool) -> Result<NaiveDate, GameError> {
    Ok(daily_repository::today(pool).await?)
}

/// The caller's game of that day, and whether this call created it. Today's layout is drawn on
/// first sight; a past day without one, or a day to come, has no daily.
pub async fn start(
    pool: &PgPool,
    user_id: Uuid,
    date: NaiveDate,
) -> Result<(Game, bool), GameError> {
    if let Some(game) = game_repository::find_daily(pool, user_id, date).await? {
        return Ok((game, false));
    }
    let today = daily_repository::today(pool).await?;
    if date > today {
        return Err(GameError::DailyNotFound);
    }
    if date == today {
        // The thread-local rng is not `Send`: it must be gone before the first await.
        let layout = SealedLayout::new(engine::draw_layout(&mut rand::rng())?);
        daily_repository::insert_if_absent(pool, date, &layout).await?;
    }
    if let Some(game) = game_repository::insert_daily(pool, user_id, date).await? {
        return Ok((game, true));
    }
    // Nothing inserted: either the day has no daily, or a racing call created the game.
    game_repository::find_daily(pool, user_id, date)
        .await?
        .map(|game| (game, false))
        .ok_or(GameError::DailyNotFound)
}
