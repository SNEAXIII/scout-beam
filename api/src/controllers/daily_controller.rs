//! `/daily/*` — the game of the day, open to guests.

use axum::Json;
use axum::extract::{Path, State};
use chrono::NaiveDate;
use http::StatusCode;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::auth::Identity;
use crate::dto::daily_dto::{DailyTodayResponse, DayRankingResponse, GlobalRankingResponse};
use crate::dto::game_dto::GameView;
use crate::errors::game_error::GameError;
use crate::extractors::current_user_extractor::CurrentUser;
use crate::services::{daily_service, game_service};
use crate::shared::error::ApiResult;
use crate::shared::openapi::ErrorResponse;
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_daily_today))
        .routes(routes!(get_daily_game, post_daily_game))
        .routes(routes!(get_daily_ranking))
        .routes(routes!(get_ranking))
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

/// `GET /daily/{date}/game` — the caller's game of that day, if they started it.
#[utoipa::path(
    get,
    path = "/daily/{date}/game",
    tag = "Daily",
    params(("date" = NaiveDate, Path, description = "Day of the daily, in Paris")),
    security((), ("bearer" = [])),
    responses(
        (status = 200, description = "The caller's game of that day", body = GameView),
        (status = 404, description = "Signed out, or not started that day", body = ErrorResponse),
    ),
)]
async fn get_daily_game(
    State(state): State<AppState>,
    caller: Option<Identity>,
    Path(date): Path<NaiveDate>,
) -> ApiResult<Json<GameView>> {
    let caller = caller.ok_or(GameError::NotFound)?;
    let game = daily_service::find(&state.pool, caller.user_id, date).await?;
    let shots = game_service::shots_of(&state.pool, &game).await?;
    Ok(Json(GameView::new(&game, shots)))
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
    let shots = game_service::shots_of(&state.pool, &game).await?;
    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(GameView::new(&game, shots))))
}

/// `GET /daily/{date}/ranking` — that day's ranking by score.
#[utoipa::path(
    get,
    path = "/daily/{date}/ranking",
    tag = "Daily",
    params(("date" = NaiveDate, Path, description = "Day of the daily, in Paris")),
    security((), ("bearer" = [])),
    description = "Solved dailies of that day, started on that same day, by registered players. With a token, `me` is the caller's line even outside the top.",
    responses(
        (status = 200, description = "The top 100, and the caller's line", body = DayRankingResponse),
    ),
)]
async fn get_daily_ranking(
    State(state): State<AppState>,
    caller: Option<Identity>,
    Path(date): Path<NaiveDate>,
) -> ApiResult<Json<DayRankingResponse>> {
    let caller = caller.map(|identity| identity.user_id);
    let ranking = daily_service::day_ranking(&state.pool, date, caller).await?;
    Ok(Json(ranking.into()))
}

/// `GET /ranking` — every registered player's points over the dailies they solved.
#[utoipa::path(
    get,
    path = "/ranking",
    tag = "Daily",
    security((), ("bearer" = [])),
    description = "Points summed over every solved daily, late ones included. With a token, `me` is the caller's line even outside the top.",
    responses(
        (status = 200, description = "The top 100, and the caller's line", body = GlobalRankingResponse),
    ),
)]
async fn get_ranking(
    State(state): State<AppState>,
    caller: Option<Identity>,
) -> ApiResult<Json<GlobalRankingResponse>> {
    let caller = caller.map(|identity| identity.user_id);
    let ranking = daily_service::global_ranking(&state.pool, caller).await?;
    Ok(Json(ranking.into()))
}
