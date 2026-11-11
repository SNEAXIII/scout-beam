//! Game domain: one attempt by a player to find a hidden layout, and the shots fired at it.
//!
//! Reaches the engine for drawing and tracing, and the user domain only through `CurrentUser`.

pub mod dto;
pub mod error;
pub mod model;
pub mod repository;
pub mod routes;
pub mod service;

pub use error::GameError;
pub use model::Game;

use axum::Router;
use utoipa::OpenApi;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().nest("/games", routes::core::router())
}

/// This domain's slice of the OpenAPI document.
#[derive(OpenApi)]
#[openapi(
    paths(
        routes::core::create_game,
        routes::core::list_games,
        routes::core::read_game,
        routes::core::fire_shot,
    ),
    components(schemas(
        dto::GameView,
        dto::GameSummaryResponse,
        dto::ShotRequest,
        dto::ShotResponse,
        model::GameStatus,
        model::ShotColour,
    )),
    tags((name = "Games", description = "Games against the bot, and the shots fired at them")),
)]
pub struct GameApi;

pub fn openapi() -> utoipa::openapi::OpenApi {
    GameApi::openapi()
}
