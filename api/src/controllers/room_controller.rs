//! `/rooms/*` — where two registered players meet before a game against each other.

use axum::Json;
use axum::extract::{Path, State};
use http::StatusCode;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::dto::room_dto::{CreateRoomRequest, HideLayoutRequest, RoomView};
use crate::engine::Layout;
use crate::errors::room_error::RoomError;
use crate::extractors::current_user_extractor::CurrentUser;
use crate::models::user_model::User;
use crate::services::game_websocket_hub::WebSocketPush;
use crate::services::room_service;
use crate::shared::error::ApiResult;
use crate::shared::openapi::ErrorResponse;
use crate::shared::validation::ValidatedJson;
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(post_room, get_rooms))
        .routes(routes!(get_room))
        .routes(routes!(post_room_join))
        .routes(routes!(post_room_layout))
}

/// `POST /rooms` — a waiting room hosted by the caller.
#[utoipa::path(
    post,
    path = "/rooms",
    tag = "Rooms",
    request_body = CreateRoomRequest,
    security(("bearer" = [])),
    responses(
        (status = 201, description = "Room opened, waiting for an opponent", body = RoomView),
        (status = 401, description = "Missing, invalid or expired token", body = ErrorResponse),
        (status = 403, description = "A guest may only play the daily", body = ErrorResponse),
        (status = 422, description = "Unknown mode", body = ErrorResponse),
    ),
)]
async fn post_room(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    ValidatedJson(body): ValidatedJson<CreateRoomRequest>,
) -> ApiResult<(StatusCode, Json<RoomView>)> {
    let seen = room_service::create(&state.pool, registered(&user)?, body.mode).await?;
    Ok((StatusCode::CREATED, Json(seen.into())))
}

/// `GET /rooms` — the caller's rooms still without a game, so they can go back to them.
#[utoipa::path(
    get,
    path = "/rooms",
    tag = "Rooms",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "Waiting or composing rooms, newest first", body = [RoomView]),
        (status = 401, description = "Missing, invalid or expired token", body = ErrorResponse),
        (status = 403, description = "A guest may only play the daily", body = ErrorResponse),
    ),
)]
async fn get_rooms(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> ApiResult<Json<Vec<RoomView>>> {
    let rooms = room_service::open_rooms(&state.pool, registered(&user)?).await?;
    Ok(Json(rooms.into_iter().map(RoomView::from).collect()))
}

/// `GET /rooms/{code}` — the room; a non-member sees only what the link needs to offer joining.
#[utoipa::path(
    get,
    path = "/rooms/{code}",
    tag = "Rooms",
    params(("code" = String, Path, description = "Room code")),
    security(("bearer" = [])),
    responses(
        (status = 200, description = "The room", body = RoomView),
        (status = 401, description = "Missing, invalid or expired token", body = ErrorResponse),
        (status = 403, description = "A guest may only play the daily", body = ErrorResponse),
        (status = 404, description = "No room under that code", body = ErrorResponse),
    ),
)]
async fn get_room(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(code): Path<String>,
) -> ApiResult<Json<RoomView>> {
    let seen = room_service::see(&state.pool, &code, registered(&user)?).await?;
    Ok(Json(seen.into()))
}

/// `POST /rooms/{code}/join` — take the free seat; a Race starts at once.
#[utoipa::path(
    post,
    path = "/rooms/{code}/join",
    tag = "Rooms",
    params(("code" = String, Path, description = "Room code")),
    security(("bearer" = [])),
    responses(
        (status = 200, description = "Seated: a Race is playing, a Duel composing", body = RoomView),
        (status = 401, description = "Missing, invalid or expired token", body = ErrorResponse),
        (status = 403, description = "A guest may only play the daily", body = ErrorResponse),
        (status = 404, description = "No room under that code", body = ErrorResponse),
        (status = 409, description = "The room is full, or the caller hosts it", body = ErrorResponse),
    ),
)]
async fn post_room_join(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(code): Path<String>,
) -> ApiResult<Json<RoomView>> {
    let seen = room_service::join(&state.pool, &code, registered(&user)?).await?;
    state
        .websocket_hub
        .publish(seen.room.id, WebSocketPush::default());
    Ok(Json(seen.into()))
}

/// `POST /rooms/{code}/layout` — hide the caller's layout in a composing Duel; the second one
/// starts both games.
#[utoipa::path(
    post,
    path = "/rooms/{code}/layout",
    tag = "Rooms",
    params(("code" = String, Path, description = "Room code")),
    request_body = HideLayoutRequest,
    security(("bearer" = [])),
    responses(
        (status = 201, description = "Layout hidden: still composing, or playing once both are", body = RoomView),
        (status = 401, description = "Missing, invalid or expired token", body = ErrorResponse),
        (status = 403, description = "A guest may only play the daily", body = ErrorResponse),
        (status = 404, description = "No room under that code", body = ErrorResponse),
        (status = 409, description = "Not a player of the room, the room is not a composing Duel, or the caller's layout is already hidden", body = ErrorResponse),
        (status = 422, description = "The layout breaks a rule, named in `detail`: the five shapes apart inside the grid, red, blue and yellow at least once", body = ErrorResponse),
    ),
)]
async fn post_room_layout(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(code): Path<String>,
    ValidatedJson(body): ValidatedJson<HideLayoutRequest>,
) -> ApiResult<(StatusCode, Json<RoomView>)> {
    let layout = Layout {
        pieces: body.pieces,
    };
    let seen = room_service::hide(&state.pool, &code, registered(&user)?, layout).await?;
    state
        .websocket_hub
        .publish(seen.room.id, WebSocketPush::default());
    Ok((StatusCode::CREATED, Json(seen.into())))
}

/// Every room route is for registered players only.
pub(crate) const fn registered(user: &User) -> Result<Uuid, RoomError> {
    if user.guest {
        Err(RoomError::AccountRequired)
    } else {
        Ok(user.id)
    }
}
