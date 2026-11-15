//! HTTP surface: each controller registers its routes with their OpenAPI description.
//!
//! Paths are written in full in each `#[utoipa::path]`, which is what mounts them: nothing is nested.

pub mod daily_controller;
pub mod game_controller;
pub mod game_websocket_controller;
pub mod user_controller;

use utoipa_axum::router::OpenApiRouter;

use crate::state::AppState;

pub fn router(dev_login_enabled: bool) -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .merge(user_controller::router(dev_login_enabled))
        .merge(game_controller::router())
        .merge(game_websocket_controller::router())
        .merge(daily_controller::router())
}
