//! What every handler is handed: the pool, the signing keys, the config, the games' WebSocket hub,
//! and the HTTP client that talks to the sign-in providers.
//!
//! Cloned per request, so each field is cheap to clone.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::PgPool;

use crate::auth::jwt::JwtCodec;
use crate::config::Config;
use crate::services::game_websocket_hub::GameWebSocketHub;

/// Per provider call; `oauth_service` bounds the whole exchange under the request timeout.
const PROVIDER_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt: JwtCodec,
    pub config: Arc<Config>,
    pub websocket_hub: GameWebSocketHub,
    pub http: reqwest::Client,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config) -> Result<Self> {
        let jwt = JwtCodec::new(
            config.jwt_secret.expose(),
            config.access_token_ttl,
            config.refresh_token_ttl,
        );
        let http = reqwest::Client::builder()
            .timeout(PROVIDER_TIMEOUT)
            .build()
            .context("cannot build the HTTP client")?;

        Ok(Self {
            pool,
            jwt,
            config: Arc::new(config),
            websocket_hub: GameWebSocketHub::default(),
            http,
        })
    }
}
