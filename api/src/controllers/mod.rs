//! HTTP surface: each controller registers its routes with their OpenAPI description.
//!
//! Paths are written in full in each `#[utoipa::path]`, which is what mounts them: nothing is nested.

pub mod game;
pub mod game_live;
pub mod user;

use utoipa_axum::router::OpenApiRouter;

use crate::state::AppState;

pub fn router(dev_login_enabled: bool) -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .merge(user::router(dev_login_enabled))
        .merge(game::router())
        .merge(game_live::router())
}
