//! Rules of a room: two players meet through a code, then play one game each.

use rand::RngExt;
use sqlx::PgPool;
use uuid::Uuid;

use crate::engine::{self, Layout};
use crate::errors::room_error::RoomError;
use crate::errors::user_error::is_unique_violation;
use crate::models::game_model::SealedLayout;
use crate::models::room_model::{Room, RoomMode, RoomStatus};
use crate::repositories::{game_repository, room_repository};

/// No `0 O 1 I`: a code is read aloud and typed by hand.
const CODE_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const CODE_LENGTH: usize = 6;
/// 32^6 codes: ten collisions in a row means something else is wrong.
const CODE_ATTEMPTS: u8 = 10;

/// A room as the caller sees it: whether they play in it, and their game once it exists.
#[derive(Debug, Clone)]
pub struct Seen {
    pub room: Room,
    pub member: bool,
    pub game_id: Option<Uuid>,
}

/// A waiting room hosted by the caller, under a code no other room holds.
pub async fn create(pool: &PgPool, host_id: Uuid, mode: RoomMode) -> Result<Seen, RoomError> {
    let mut attempts = 1;
    let code = loop {
        let code = draw_code();
        match room_repository::insert(pool, &code, mode, host_id).await {
            Err(error) if is_unique_violation(&error) && attempts < CODE_ATTEMPTS => attempts += 1,
            inserted => break inserted.map(|()| code)?,
        }
    };
    see(pool, &code, host_id).await
}

pub async fn see(pool: &PgPool, code: &str, caller: Uuid) -> Result<Seen, RoomError> {
    let room = find(pool, code).await?;
    let member = room.has_player(caller);
    let game_id = if member {
        room_repository::game_of(pool, room.id, caller).await?
    } else {
        None
    };
    Ok(Seen {
        room,
        member,
        game_id,
    })
}

/// The caller's rooms not playing yet, newest first.
pub async fn open_rooms(pool: &PgPool, caller: Uuid) -> Result<Vec<Seen>, RoomError> {
    let rooms = room_repository::list_open_for(pool, caller).await?;
    Ok(rooms
        .into_iter()
        .map(|room| Seen {
            room,
            member: true,
            game_id: None,
        })
        .collect())
}

/// Take the free seat. A Race starts at once, both games on one layout; a Duel is composed first.
pub async fn join(pool: &PgPool, code: &str, caller: Uuid) -> Result<Seen, RoomError> {
    let room = find(pool, code).await?;
    let (status, layout) = match room.mode {
        RoomMode::Race => (
            RoomStatus::Playing,
            Some(SealedLayout::new(engine::draw_layout(&mut rand::rng())?)),
        ),
        RoomMode::Duel => (RoomStatus::Composing, None),
    };

    let mut tx = pool.begin().await?;
    let Some(room_id) = room_repository::seat_opponent(&mut *tx, code, caller, status).await?
    else {
        tx.rollback().await?;
        return Err(if room.host_id == caller {
            RoomError::OwnRoom
        } else {
            RoomError::Full
        });
    };
    if let Some(layout) = layout {
        game_repository::insert_in_room(&mut *tx, room.host_id, room_id, &layout).await?;
        game_repository::insert_in_room(&mut *tx, caller, room_id, &layout).await?;
    }
    tx.commit().await?;
    see(pool, code, caller).await
}

/// Hide the caller's layout in a composing Duel; the second one starts both games, each player
/// hunting the layout the other hid.
pub async fn hide(
    pool: &PgPool,
    code: &str,
    caller: Uuid,
    layout: Layout,
) -> Result<Seen, RoomError> {
    let room = find(pool, code).await?;
    if !room.has_player(caller) {
        return Err(RoomError::NotAPlayer);
    }
    if room.status != RoomStatus::Composing {
        return Err(RoomError::NotComposing);
    }
    layout.validate_hidden()?;

    let mut tx = pool.begin().await?;
    let hidden =
        room_repository::hide_layout(&mut *tx, room.id, caller, &SealedLayout::new(layout)).await?;
    match hidden {
        None => return Err(RoomError::LayoutAlreadyHidden),
        Some((Some(host_layout), Some(opponent_layout))) => {
            let opponent_id = room.opponent_id.ok_or(RoomError::NotComposing)?;
            room_repository::start(&mut *tx, room.id).await?;
            game_repository::insert_in_room(&mut *tx, room.host_id, room.id, &opponent_layout)
                .await?;
            game_repository::insert_in_room(&mut *tx, opponent_id, room.id, &host_layout).await?;
        }
        Some(_) => {}
    }
    tx.commit().await?;
    see(pool, code, caller).await
}

async fn find(pool: &PgPool, code: &str) -> Result<Room, RoomError> {
    room_repository::find_by_code(pool, code)
        .await?
        .ok_or(RoomError::NotFound)
}

fn draw_code() -> String {
    let mut rng = rand::rng();
    // Indexing a constant, non-empty array: no Option to unwrap, no panic path.
    (0..CODE_LENGTH)
        .map(|_| char::from(CODE_ALPHABET[rng.random_range(0..CODE_ALPHABET.len())]))
        .collect()
}
