//! What every handler is handed: the pool, the signing keys, the config, the game sockets' hub.
//!
//! Cloned per request, so each field is cheap to clone.

use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::PgPool;

use crate::auth::jwt::JwtCodec;
use crate::config::Config;
use crate::domains::game::hub::GameHub;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt: JwtCodec,
    pub config: Arc<Config>,
    pub hub: GameHub,
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
            hub: GameHub::default(),
        }
    }
}

/// Lets `Identity` be extracted from any state that can produce a codec.
impl FromRef<AppState> for JwtCodec {
    fn from_ref(state: &AppState) -> Self {
        state.jwt.clone()
    }
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}
