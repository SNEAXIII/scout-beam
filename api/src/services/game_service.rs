//! Rules of a game: controllers never reach its repository but through here.

use sqlx::PgPool;
use uuid::Uuid;

use crate::engine::{self, Entry, Layout};
use crate::errors::game_error::GameError;
use crate::models::game_model::{Game, GameStatus, GameSummary, Pace, SealedLayout, Shot};
use crate::repositories::{game_repository, room_repository};

/// Whether a shot was just fired, or had been fired before.
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
    Ok(game_repository::insert(pool, user_id, &SealedLayout::new(layout)).await?)
}

pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<GameSummary>, GameError> {
    Ok(game_repository::list_for_user(pool, user_id).await?)
}

/// The caller's game; another player's answers `NotFound`, exactly like a missing one.
pub async fn get_owned(pool: &PgPool, game_id: Uuid, user_id: Uuid) -> Result<Game, GameError> {
    game_repository::find_owned(pool, game_id, user_id)
        .await?
        .ok_or(GameError::NotFound)
}

/// A room game's pace; `None` outside a room.
pub async fn pace_of(pool: &PgPool, game: &Game) -> Result<Option<Pace>, GameError> {
    if game.room_id.is_none() {
        return Ok(None);
    }
    Ok(game_repository::pace_of(pool, game.id).await?)
}

/// Shots in firing order.
pub async fn shots_of(pool: &PgPool, game: &Game) -> Result<Vec<Shot>, GameError> {
    Ok(game_repository::shots_of(pool, game.id).await?)
}

/// Trace a beam from `entry` and store it, unless that entry was already fired.
///
/// In a room a new shot spends the allowance, and the pace comes along, this shot counted.
/// A shot racing the end of the game is harmless: the layout is revealed then anyway.
pub async fn fire(
    pool: &PgPool,
    game: &Game,
    entry: Entry,
) -> Result<(Shot, Firing, Option<Pace>), GameError> {
    playable(game.status)?;

    let traced = engine::trace(game.layout.pieces(), entry)?;
    let shot = Shot {
        entry: i16::from(traced.entry.number()),
        exit_entry: i16::from(traced.exit.number()),
        colour: traced.beam_colour.into(),
    };

    let mut tx = pool.begin().await?;
    // Locked on its own first: the pace read next then sees a racing tab's committed shot.
    if game.room_id.is_some() {
        game_repository::lock_status(&mut *tx, game.id).await?;
    }
    let before = game_repository::pace_of(&mut *tx, game.id).await?;
    if let Some(stored) = game_repository::insert_shot_if_absent(&mut *tx, game.id, &shot).await? {
        if let Some(pace) = before
            && pace.shots_left() <= 0
        {
            tx.rollback().await?;
            return Err(GameError::NoShotLeft {
                retry_after_ms: pace.next_shot_in_ms(),
            });
        }
        let pace = game_repository::pace_of(&mut *tx, game.id).await?;
        tx.commit().await?;
        return Ok((stored, Firing::New, pace));
    }
    tx.rollback().await?;
    game_repository::find_shot(pool, game.id, shot.entry)
        .await?
        .map(|stored| (stored, Firing::Replayed, before))
        .ok_or(GameError::NotFound)
}

/// Store a valid proposition, once a room's cooldown is over; a correct one solves the game in the
/// same transaction, and wins its room. The pace comes along in a room, this proposition counted.
pub async fn propose(
    pool: &PgPool,
    game: &Game,
    layout: &Layout,
) -> Result<(Verdict, Option<Pace>), GameError> {
    playable(game.status)?;
    layout.validate()?;
    let correct = engine::same_shots(game.layout.pieces(), layout)?;

    let mut tx = pool.begin().await?;
    // Room before game, here and in `surrender`: both players' endings lock in the same order.
    let room = room_repository::lock_for_game(&mut *tx, game.id).await?;
    // The lock holds a racing proposition or surrender until this one commits; it then reads the end.
    let status = game_repository::lock_status(&mut *tx, game.id).await?;
    if let Err(refusal) = status.ok_or(GameError::NotFound).and_then(playable) {
        tx.rollback().await?;
        return Err(refusal);
    }
    if let Some(pace) = game_repository::pace_of(&mut *tx, game.id).await?
        && pace.proposition_in_ms() > 0
    {
        tx.rollback().await?;
        return Err(GameError::PropositionCooldown {
            retry_after_ms: pace.proposition_in_ms(),
        });
    }
    game_repository::insert_proposition(&mut *tx, game.id, layout, correct).await?;
    let pace = game_repository::pace_of(&mut *tx, game.id).await?;
    let verdict = if correct {
        let solved = game_repository::mark_solved(&mut *tx, game.id).await?;
        if let Some(room) = room {
            room_repository::finish(&mut *tx, room.id, game.user_id).await?;
        }
        Verdict::Correct(solved)
    } else {
        Verdict::Wrong
    };
    tx.commit().await?;
    Ok((verdict, pace))
}

/// Give up: the game ends unsolved and its layout is revealed; in a room, the opponent wins.
/// A daily is never given up.
pub async fn surrender(pool: &PgPool, game: &Game) -> Result<Game, GameError> {
    if game.daily_date.is_some() {
        return Err(GameError::DailySurrender);
    }
    playable(game.status)?;

    let mut tx = pool.begin().await?;
    // Same locks as `propose`: whichever of the two commits first ends the game.
    let room = room_repository::lock_for_game(&mut *tx, game.id).await?;
    let status = game_repository::lock_status(&mut *tx, game.id).await?;
    if let Err(refusal) = status.ok_or(GameError::NotFound).and_then(playable) {
        tx.rollback().await?;
        return Err(refusal);
    }
    let surrendered = game_repository::mark_surrendered(&mut *tx, game.id).await?;
    if let Some(room) = room {
        room_repository::finish(&mut *tx, room.id, room.opponent_id).await?;
    }
    tx.commit().await?;
    Ok(surrendered)
}

/// Only a game in progress takes a shot, a proposition or a surrender.
const fn playable(status: GameStatus) -> Result<(), GameError> {
    match status {
        GameStatus::InProgress => Ok(()),
        GameStatus::Solved => Err(GameError::AlreadySolved),
        GameStatus::Surrendered => Err(GameError::Surrendered),
        GameStatus::Lost => Err(GameError::Lost),
    }
}

/// The entry rule is the engine's; this only puts it in the game's vocabulary.
pub fn entry(number: u8) -> Result<Entry, GameError> {
    Entry::new(number).map_err(|_| GameError::InvalidEntry)
}
