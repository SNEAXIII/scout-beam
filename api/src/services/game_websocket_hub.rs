//! Fan-out of a game's or a room's events to every socket open on it, so another tab sees a shot
//! or the end, and the opponent a shot count.
//!
//! In memory: the stack runs a single API replica.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use tokio::sync::broadcast;
use uuid::Uuid;

use crate::dto::game_dto::WebSocketServerMessage;

/// A socket further behind than this gets a fresh snapshot instead of the events it missed.
const CAPACITY: usize = 64;

/// An event, with who already knows it: the socket it came from, or the player it is about.
/// The default is a bare change: a room's lobby sockets read it again.
#[derive(Debug, Clone, Default)]
pub struct WebSocketPush {
    pub origin: Option<Uuid>,
    /// `None`: the game changed, each socket sends its own snapshot.
    pub message: Option<WebSocketServerMessage>,
}

#[derive(Debug, Clone, Default)]
pub struct GameWebSocketHub {
    channels: Arc<Mutex<HashMap<Uuid, broadcast::Sender<WebSocketPush>>>>,
}

impl GameWebSocketHub {
    pub fn subscribe(&self, key: Uuid) -> broadcast::Receiver<WebSocketPush> {
        self.lock()
            .entry(key)
            .or_insert_with(|| broadcast::channel(CAPACITY).0)
            .subscribe()
    }

    /// With no socket open on the game or room, nobody is told and nothing is kept.
    pub fn publish(&self, key: Uuid, push: WebSocketPush) {
        if let Some(sender) = self.lock().get(&key) {
            // Fails only when every receiver is gone, which leaves nobody to tell.
            let _ = sender.send(push);
        }
    }

    /// Called by a closing socket once its receiver is dropped.
    pub fn release(&self, key: Uuid) {
        let mut channels = self.lock();
        if channels
            .get(&key)
            .is_some_and(|sender| sender.receiver_count() == 0)
        {
            channels.remove(&key);
        }
    }

    /// No code panics while holding the lock, so a poisoned map is still consistent.
    fn lock(&self) -> MutexGuard<'_, HashMap<Uuid, broadcast::Sender<WebSocketPush>>> {
        self.channels.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
