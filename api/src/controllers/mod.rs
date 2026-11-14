//! HTTP surface: each controller declares its routes, mounted here under their prefix.

pub mod auth;
pub mod game;
pub mod game_live;

use axum::Router;

use crate::state::AppState;

pub fn router(dev_login_enabled: bool) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router(dev_login_enabled))
        .nest("/games", game::router().merge(game_live::router()))
}
