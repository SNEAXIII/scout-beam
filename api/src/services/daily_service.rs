//! Rules of the daily: one layout a day for every player, one game each.

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::engine;
use crate::errors::game_error::GameError;
use crate::models::daily_model::{DayRankingRow, GlobalRankingRow};
use crate::models::game_model::{Game, SealedLayout};
use crate::repositories::{daily_repository, game_repository};

pub async fn today(pool: &PgPool) -> Result<NaiveDate, GameError> {
    Ok(daily_repository::today(pool).await?)
}

/// The caller's game of that day, without starting it.
pub async fn find(pool: &PgPool, user_id: Uuid, date: NaiveDate) -> Result<Game, GameError> {
    game_repository::find_daily(pool, user_id, date)
        .await?
        .ok_or(GameError::NotFound)
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

/// The first 100 rows, and the caller's own wherever it falls; `me` is `None` when unranked.
#[derive(Debug, Clone)]
pub struct Ranking<T> {
    pub top: Vec<T>,
    pub me: Option<T>,
}

pub async fn day_ranking(
    pool: &PgPool,
    date: NaiveDate,
    caller: Option<Uuid>,
) -> Result<Ranking<DayRankingRow>, GameError> {
    let rows = daily_repository::day_ranking(pool, date, caller).await?;
    Ok(split(rows, |row| row.in_top, |row| row.mine))
}

pub async fn global_ranking(
    pool: &PgPool,
    caller: Option<Uuid>,
) -> Result<Ranking<GlobalRankingRow>, GameError> {
    let rows = daily_repository::global_ranking(pool, caller).await?;
    Ok(split(rows, |row| row.in_top, |row| row.mine))
}

fn split<T: Clone>(rows: Vec<T>, in_top: fn(&T) -> bool, mine: fn(&T) -> bool) -> Ranking<T> {
    let me = rows.iter().find(|row| mine(row)).cloned();
    let top = rows.into_iter().filter(in_top).collect();
    Ranking { top, me }
}
