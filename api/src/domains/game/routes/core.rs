//! `/games/*` over HTTP — starting games and reading them; playing one goes through its socket.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use http::StatusCode;
use uuid::Uuid;

use crate::domains::game::dto::{GameSummaryResponse, GameView};
use crate::domains::game::service;
use crate::domains::user::CurrentUser;
use crate::shared::error::ApiResult;
#[allow(unused_imports)] // referenced by the utoipa annotations
use crate::shared::openapi::ErrorResponse;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // One call per path: registering the same path twice makes axum panic at startup.
        .route("/", post(create_game).get(list_games))
        .route("/{game_id}", get(read_game))
}

/// `POST /games` — the bot draws a hidden layout.
#[utoipa::path(
    post,
    path = "/games",
    tag = "Games",
    security(("bearer" = [])),
    responses(
        (status = 201, description = "Game started, layout hidden", body = GameView),
        (status = 401, description = "Missing, invalid or expired token", body = ErrorResponse),
    ),
)]
pub(crate) async fn create_game(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> ApiResult<(StatusCode, Json<GameView>)> {
    let game = service::create(&state.pool, user.id).await?;
    Ok((StatusCode::CREATED, Json(GameView::new(&game, Vec::new()))))
}

/// `GET /games` — the caller's games, newest first.
#[utoipa::path(
    get,
    path = "/games",
    tag = "Games",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "The caller's games, newest first", body = [GameSummaryResponse]),
        (status = 401, description = "Missing, invalid or expired token", body = ErrorResponse),
    ),
)]
pub(crate) async fn list_games(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> ApiResult<Json<Vec<GameSummaryResponse>>> {
    let games = service::list(&state.pool, user.id).await?;
    Ok(Json(games.into_iter().map(Into::into).collect()))
}

/// `GET /games/{game_id}` — status and shots; the layout only once the game is over.
#[utoipa::path(
    get,
    path = "/games/{game_id}",
    tag = "Games",
    params(("game_id" = Uuid, Path, description = "Game")),
    security(("bearer" = [])),
    responses(
        (status = 200, description = "The game, its shots in firing order", body = GameView),
        (status = 401, description = "Missing, invalid or expired token", body = ErrorResponse),
        (status = 404, description = "No such game among the caller's", body = ErrorResponse),
    ),
)]
pub(crate) async fn read_game(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(game_id): Path<Uuid>,
) -> ApiResult<Json<GameView>> {
    let game = service::get_owned(&state.pool, game_id, user.id).await?;
    let shots = service::shots_of(&state.pool, &game).await?;
    Ok(Json(GameView::new(&game, shots)))
}
