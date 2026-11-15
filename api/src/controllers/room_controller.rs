//! `/rooms/*` — where two registered players meet before a game against each other.

use axum::Json;
use axum::extract::{Path, State};
use http::StatusCode;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::dto::room_dto::{CreateRoomRequest, RoomView};
use crate::errors::room_error::RoomError;
use crate::extractors::current_user_extractor::CurrentUser;
use crate::models::user_model::User;
use crate::services::room_service;
use crate::shared::error::ApiResult;
use crate::shared::openapi::ErrorResponse;
use crate::shared::validation::ValidatedJson;
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(post_room))
        .routes(routes!(get_room))
        .routes(routes!(post_room_join))
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
    Ok(Json(seen.into()))
}

/// Every room route is for registered players only.
const fn registered(user: &User) -> Result<Uuid, RoomError> {
    if user.guest {
        Err(RoomError::AccountRequired)
    } else {
        Ok(user.id)
    }
}
