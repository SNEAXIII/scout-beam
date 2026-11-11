//! Rules of the game domain — the only door other domains may use.

use sqlx::PgPool;
use uuid::Uuid;

use crate::domains::game::error::GameError;
use crate::domains::game::model::{Game, GameStatus, GameSummary, SealedLayout, Shot};
use crate::domains::game::repository;
use crate::engine::{self, Entry, Layout};

/// Whether a shot was just fired or had been fired before.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Firing {
    New,
    Replayed,
}

/// What a stored proposition earned: the game, now solved, when it was correct.
#[derive(Debug, Clone)]
pub enum Verdict {
    Wrong,
    Correct(Game),
}

/// The bot draws a layout and the game starts with no shot.
pub async fn create(pool: &PgPool, user_id: Uuid) -> Result<Game, GameError> {
    // The thread-local rng is not `Send`: it must be gone before the first await.
    let layout = engine::draw_layout(&mut rand::rng())?;
    Ok(repository::insert(pool, user_id, &SealedLayout::new(layout)).await?)
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<GameSummary>, GameError> {
    Ok(repository::list_for_user(pool, user_id).await?)
}

/// The caller's game; another player's answers `NotFound`, exactly like a missing one.
pub async fn get_owned(pool: &PgPool, game_id: Uuid, user_id: Uuid) -> Result<Game, GameError> {
    repository::find_owned(pool, game_id, user_id)
        .await?
        .ok_or(GameError::NotFound)
}

/// Shots in firing order.
pub async fn shots_of(pool: &PgPool, game: &Game) -> Result<Vec<Shot>, GameError> {
    Ok(repository::shots_of(pool, game.id).await?)
}

/// Trace a beam from `entry` and store it, unless that entry was already fired.
/// A shot racing the end of the game is harmless: the layout is revealed then anyway.
pub async fn fire(pool: &PgPool, game: &Game, entry: Entry) -> Result<(Shot, Firing), GameError> {
    playable(game.status)?;

    let traced = engine::trace(game.layout.pieces(), entry)?;
    let shot = Shot {
        entry: i16::from(traced.entry.number()),
        exit_entry: i16::from(traced.exit.number()),
        colour: traced.beam_colour.into(),
    };

    if let Some(stored) = repository::insert_shot_if_absent(pool, game.id, &shot).await? {
        return Ok((stored, Firing::New));
    }
    repository::find_shot(pool, game.id, shot.entry)
        .await?
        .map(|stored| (stored, Firing::Replayed))
        .ok_or(GameError::NotFound)
}

/// Store a valid proposition; a correct one solves the game in the same transaction.
pub async fn propose(pool: &PgPool, game: &Game, layout: &Layout) -> Result<Verdict, GameError> {
    playable(game.status)?;
    layout.validate()?;
    let correct = engine::same_shots(game.layout.pieces(), layout)?;

    let mut tx = pool.begin().await?;
    // The lock holds a racing proposition or surrender until this one commits; it then reads the end.
    let status = repository::lock_status(&mut *tx, game.id).await?;
    if let Err(refusal) = status.ok_or(GameError::NotFound).and_then(playable) {
        tx.rollback().await?;
        return Err(refusal);
    }
    repository::insert_proposition(&mut *tx, game.id, layout, correct).await?;
    let verdict = if correct {
        Verdict::Correct(repository::mark_solved(&mut *tx, game.id).await?)
    } else {
        Verdict::Wrong
    };
    tx.commit().await?;
    Ok(verdict)
}

/// Give up: the game ends unsolved and its layout is revealed.
pub async fn surrender(pool: &PgPool, game: &Game) -> Result<Game, GameError> {
    playable(game.status)?;

    let mut tx = pool.begin().await?;
    // Same lock as `propose`: whichever of the two commits first ends the game.
    let status = repository::lock_status(&mut *tx, game.id).await?;
    if let Err(refusal) = status.ok_or(GameError::NotFound).and_then(playable) {
        tx.rollback().await?;
        return Err(refusal);
    }
    let surrendered = repository::mark_surrendered(&mut *tx, game.id).await?;
    tx.commit().await?;
    Ok(surrendered)
}

/// Only a game in progress takes a shot, a proposition or a surrender.
const fn playable(status: GameStatus) -> Result<(), GameError> {
    match status {
        GameStatus::InProgress => Ok(()),
        GameStatus::Solved => Err(GameError::AlreadySolved),
        GameStatus::Surrendered => Err(GameError::Surrendered),
    }
}

/// The entry rule is the engine's; this only puts it in the game's vocabulary.
pub fn entry(number: u8) -> Result<Entry, GameError> {
    Entry::new(number).map_err(|_| GameError::InvalidEntry)
}
