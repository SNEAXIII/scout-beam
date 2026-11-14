//! What every handler is handed: the pool, the signing keys, the config, the games' WebSocket hub.
//!
//! Cloned per request, so each field is cheap to clone.

use std::sync::Arc;

use sqlx::PgPool;

use crate::auth::jwt::JwtCodec;
use crate::config::Config;
use crate::services::game_websocket_hub::GameWebSocketHub;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt: JwtCodec,
    pub config: Arc<Config>,
    pub websocket_hub: GameWebSocketHub,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config) -> Self {
        let jwt = JwtCodec::new(
            &config.jwt_secret,
            config.access_token_ttl,
            config.refresh_token_ttl,
        );

        Self {
            pool,
            jwt,
            config: Arc::new(config),
            websocket_hub: GameWebSocketHub::default(),
        }
    }
}
