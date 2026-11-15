//! `/rooms/{code}/ws` — a room's lobby: a bare signal each time the room changes, so the page
//! reads it again instead of polling.

use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Response;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::sleep_until;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::auth::{AuthError, Identity};
use crate::controllers::game_websocket_controller::{
    check_origin, close_with_error, next_text_frame, send_message, token_deadline,
};
use crate::controllers::room_controller::registered;
use crate::dto::game_dto::WebSocketServerMessage;
use crate::extractors::current_user_extractor::CurrentUser;
use crate::services::room_service;
use crate::shared::error::ApiError;
use crate::shared::openapi::ErrorResponse;
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_room_websocket))
}

/// `GET /rooms/{code}/ws` — the room's lobby socket.
#[utoipa::path(
    get,
    path = "/rooms/{code}/ws",
    tag = "Rooms",
    params(("code" = String, Path, description = "Room code")),
    security((), ("bearer" = [])),
    description = "Authenticated at the upgrade like `GET /rooms/{code}`, by the access cookie or \
        a bearer header. The server sends `room` on connecting, then each time the room changes \
        (an opponent seated, a layout hidden, the games started): the signal carries nothing, the \
        client reads `GET /rooms/{code}` again. A refused connection, or the token expiring, sends \
        an `error` with action `connect` and closes with code 4000 + status.",
    responses(
        (status = 101, description = "Switched to the room's WebSocket"),
        (status = 403, description = "Opened from another origin than the front's", body = ErrorResponse),
    ),
)]
async fn get_room_websocket(
    State(state): State<AppState>,
    Path(code): Path<String>,
    headers: HeaderMap,
    caller: Result<Identity, ApiError>,
    upgrade: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    check_origin(&state, &headers)?;
    Ok(upgrade.on_upgrade(move |socket| run_websocket(socket, state, code, caller)))
}

async fn run_websocket(
    mut socket: WebSocket,
    state: AppState,
    code: String,
    caller: Result<Identity, ApiError>,
) {
    let joined = match caller {
        Ok(identity) => room_of(&state, &code, &identity)
            .await
            .map(|room_id| (room_id, identity.expires_at)),
        Err(refusal) => Err(refusal),
    };
    let (room_id, expires_at) = match joined {
        Ok(joined) => joined,
        Err(refusal) => return close_with_error(socket, refusal).await,
    };

    // Subscribed before the first signal, so no change falls between it and the client's read.
    let mut pushes = state.websocket_hub.subscribe(room_id);
    let expiry = sleep_until(token_deadline(expires_at));
    tokio::pin!(expiry);
    let mut ended: Option<ApiError> = None;
    let mut changed = true;
    loop {
        if changed
            && send_message(&mut socket, &WebSocketServerMessage::Room)
                .await
                .is_err()
        {
            break;
        }
        changed = tokio::select! {
            text = next_text_frame(&mut socket) => match text {
                Some(_) => false,
                None => break,
            },
            // A pushed message is the play's (`opponent`): only a bare push is the room's.
            event = pushes.recv() => match event {
                Ok(push) => push.message.is_none(),
                Err(RecvError::Lagged(_)) => true,
                Err(RecvError::Closed) => break,
            },
            () = &mut expiry => {
                ended = Some(AuthError::ExpiredToken.into());
                break;
            }
        };
    }
    drop(pushes);
    state.websocket_hub.release(room_id);
    if let Some(refusal) = ended {
        close_with_error(socket, refusal).await;
    }
}

/// Whoever may read the room may be told it changed: the re-read applies the same visibility.
async fn room_of(state: &AppState, code: &str, identity: &Identity) -> Result<Uuid, ApiError> {
    let CurrentUser(user) = CurrentUser::of(state, identity.user_id).await?;
    let seen = room_service::see(&state.pool, code, registered(&user)?).await?;
    Ok(seen.room.id)
}
