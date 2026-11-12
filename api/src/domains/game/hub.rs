//! Fan-out of a game's events to every socket open on it, so another tab sees a shot or the end.
//!
//! In memory: the stack runs a single API replica.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use tokio::sync::broadcast;
use uuid::Uuid;

use crate::domains::game::dto::ServerMessage;

/// A socket further behind than this gets a fresh snapshot instead of the events it missed.
const CAPACITY: usize = 64;

/// An event, with the socket it came from: that one already had it as its answer.
#[derive(Debug, Clone)]
pub struct Push {
    pub origin: Uuid,
    pub message: ServerMessage,
}

#[derive(Debug, Clone, Default)]
pub struct GameHub {
    channels: Arc<Mutex<HashMap<Uuid, broadcast::Sender<Push>>>>,
}

impl GameHub {
    pub fn subscribe(&self, game_id: Uuid) -> broadcast::Receiver<Push> {
        self.lock()
            .entry(game_id)
            .or_insert_with(|| broadcast::channel(CAPACITY).0)
            .subscribe()
    }

    /// With no socket open on the game, nobody is told and nothing is kept.
    pub fn publish(&self, game_id: Uuid, push: Push) {
        if let Some(sender) = self.lock().get(&game_id) {
            // Fails only when every receiver is gone, which leaves nobody to tell.
            let _ = sender.send(push);
        }
    }

    /// Called by a closing socket once its receiver is dropped.
    pub fn release(&self, game_id: Uuid) {
        let mut channels = self.lock();
        if channels
            .get(&game_id)
            .is_some_and(|sender| sender.receiver_count() == 0)
        {
            channels.remove(&game_id);
        }
    }

    /// No code panics while holding the lock, so a poisoned map is still consistent.
    fn lock(&self) -> MutexGuard<'_, HashMap<Uuid, broadcast::Sender<Push>>> {
        self.channels.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
