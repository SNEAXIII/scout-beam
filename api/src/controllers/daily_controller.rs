//! `/daily/*` — the game of the day, open to guests.

use axum::Json;
use axum::extract::{Path, State};
use chrono::NaiveDate;
use http::StatusCode;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::dto::daily_dto::DailyTodayResponse;
use crate::dto::game_dto::GameView;
use crate::extractors::current_user_extractor::CurrentUser;
use crate::services::{daily_service, game_service};
use crate::shared::error::ApiResult;
use crate::shared::openapi::ErrorResponse;
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_daily_today))
        .routes(routes!(post_daily_game))
}

/// `GET /daily/today` — today's date in Paris, so the front never computes it.
#[utoipa::path(
    get,
    path = "/daily/today",
    tag = "Daily",
    responses(
        (status = 200, description = "Today's daily date", body = DailyTodayResponse),
    ),
)]
async fn get_daily_today(State(state): State<AppState>) -> ApiResult<Json<DailyTodayResponse>> {
    let date = daily_service::today(&state.pool).await?;
    Ok(Json(DailyTodayResponse { date }))
}

/// `POST /daily/{date}/game` — the caller's game of that day, started on the first call.
#[utoipa::path(
    post,
    path = "/daily/{date}/game",
    tag = "Daily",
    params(("date" = NaiveDate, Path, description = "Day of the daily, in Paris")),
    security(("bearer" = [])),
    responses(
        (status = 201, description = "Game started on the day's layout", body = GameView),
        (status = 200, description = "The caller's game of that day, already started", body = GameView),
        (status = 401, description = "Missing, invalid or expired token", body = ErrorResponse),
        (status = 404, description = "No daily that day: past without one, or to come", body = ErrorResponse),
    ),
)]
async fn post_daily_game(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(date): Path<NaiveDate>,
) -> ApiResult<(StatusCode, Json<GameView>)> {
    let (game, created) = daily_service::start(&state.pool, user.id, date).await?;
    if created {
        return Ok((StatusCode::CREATED, Json(GameView::new(&game, Vec::new()))));
    }
    let shots = game_service::shots_of(&state.pool, &game).await?;
    Ok((StatusCode::OK, Json(GameView::new(&game, shots))))
}
