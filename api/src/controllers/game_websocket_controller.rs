//! `/games/{game_id}/ws` — playing a game over one socket: shots, propositions and surrender,
//! with what the player's other sockets on it change pushed in.

use std::time::Duration;

use axum::extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::http::header::ORIGIN;
use axum::response::Response;
use chrono::Utc;
use tokio::sync::broadcast::{self, error::RecvError};
use tokio::time::{Instant, sleep_until};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use crate::auth::{AuthError, Identity};
use crate::dto::game_dto::{Action, GameView, WebSocketClientMessage, WebSocketServerMessage};
use crate::engine::Layout;
use crate::errors::game_error::GameError;
use crate::extractors::current_user_extractor::CurrentUser;
use crate::models::game_model::Game;
use crate::services::game_service::{self, Firing, Verdict};
use crate::services::game_websocket_hub::WebSocketPush;
use crate::shared::error::ApiError;
use crate::shared::openapi::ErrorResponse;
use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_game_websocket))
}

/// `GET /games/{game_id}/ws` — the game's socket.
#[utoipa::path(
    get,
    path = "/games/{game_id}/ws",
    tag = "Games",
    params(("game_id" = Uuid, Path, description = "Game")),
    security((), ("bearer" = [])),
    description = "Authenticated at the upgrade, by the access cookie or a bearer header; the \
        server opens with `game`. Then `fire {entry}`, `propose {pieces}` and `surrender` are each \
        answered, in order, by `shot`, `verdict`, `game` or `error` carrying the HTTP status. The \
        player's other sockets on the game are pushed `shot` and `game`. A refused connection, or \
        the token expiring, sends an `error` with action `connect` and closes with code 4000 + status.",
    responses(
        (status = 101, description = "Switched to the game's WebSocket"),
        (status = 403, description = "Opened from another origin than the front's", body = ErrorResponse),
    ),
)]
async fn get_game_websocket(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
    headers: HeaderMap,
    caller: Result<Identity, ApiError>,
    upgrade: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    // A browser sends the cookies whichever page opens the socket: only the front may.
    // No `Origin` at all is a tool or a test, which can send any cookie it likes anyway.
    if let Some(origin) = headers.get(ORIGIN)
        && origin.as_bytes() != state.config.cors_allowed_origin.as_bytes()
    {
        return Err(ApiError::Forbidden("Origine non autorisée".into()));
    }
    // An auth refusal still upgrades: a browser reads the 4401 close code, never a 401 status.
    Ok(upgrade.on_upgrade(move |socket| run_websocket(socket, state, game_id, caller)))
}

async fn run_websocket(
    mut socket: WebSocket,
    state: AppState,
    game_id: Uuid,
    caller: Result<Identity, ApiError>,
) {
    let joined = match caller {
        Ok(identity) => join_game(&mut socket, state, game_id, identity).await,
        Err(refusal) => Err(refusal),
    };
    let (session, mut pushes, expires_at) = match joined {
        Ok(joined) => joined,
        Err(refusal) => return close_with_error(socket, refusal).await,
    };
    let ended = session.serve(&mut socket, &mut pushes, expires_at).await;
    drop(pushes);
    session.state.websocket_hub.release(game_id);
    if let Some(refusal) = ended {
        close_with_error(socket, refusal).await;
    }
}

/// Check the account still exists and the game is its own, then send the game's snapshot.
async fn join_game(
    socket: &mut WebSocket,
    state: AppState,
    game_id: Uuid,
    identity: Identity,
) -> Result<
    (
        WebSocketSession,
        broadcast::Receiver<WebSocketPush>,
        Instant,
    ),
    ApiError,
> {
    let CurrentUser(user) = CurrentUser::of(&state, identity.user_id).await?;
    game_service::get_owned(&state.pool, game_id, user.id).await?;

    // Subscribed before the snapshot is read, so no event falls between the two.
    let pushes = state.websocket_hub.subscribe(game_id);
    let session = WebSocketSession {
        state,
        game_id,
        user_id: user.id,
        id: Uuid::new_v4(),
    };
    match session.snapshot().await {
        Ok(game) => {
            // A client already gone is seen by `serve` on its first read.
            let _ = send_message(socket, &WebSocketServerMessage::Game { game }).await;
            Ok((session, pushes, token_deadline(identity.expires_at)))
        }
        Err(error) => {
            drop(pushes);
            session.state.websocket_hub.release(game_id);
            Err(error.into())
        }
    }
}

/// One authenticated socket on one game.
struct WebSocketSession {
    state: AppState,
    game_id: Uuid,
    user_id: Uuid,
    /// Tells this socket's own events apart from the other sockets'.
    id: Uuid,
}

impl WebSocketSession {
    /// Until the client leaves; `Some` when the server ends the connection.
    async fn serve(
        &self,
        socket: &mut WebSocket,
        pushes: &mut broadcast::Receiver<WebSocketPush>,
        expires_at: Instant,
    ) -> Option<ApiError> {
        let expiry = sleep_until(expires_at);
        tokio::pin!(expiry);
        loop {
            let outgoing = tokio::select! {
                text = next_text_frame(socket) => self.answer_client_message(&text?).await,
                event = pushes.recv() => match event { Ok(push) if push.origin == self.id => continue, Ok(push) => push.message, Err(RecvError::Lagged(_)) => match self.snapshot().await { Ok(game) => WebSocketServerMessage::Game { game }, Err(error) => return Some(error.into()), }, Err(RecvError::Closed) => return None, },
                () = &mut expiry => return Some(AuthError::ExpiredToken.into()),
            };
            send_message(socket, &outgoing).await.ok()?;
        }
    }

    async fn answer_client_message(&self, text: &str) -> WebSocketServerMessage {
        let message = match serde_json::from_str::<WebSocketClientMessage>(text) {
            Ok(message) => message,
            Err(cause) => {
                return error_message(
                    Action::Message,
                    ApiError::UnprocessableBody(cause.to_string()),
                );
            }
        };
        match message {
            WebSocketClientMessage::Fire { entry } => self
                .fire(entry)
                .await
                .unwrap_or_else(|refusal| error_message(Action::Fire, refusal.into())),
            WebSocketClientMessage::Propose { pieces } => self
                .propose(&Layout { pieces })
                .await
                .unwrap_or_else(|refusal| error_message(Action::Propose, refusal.into())),
            WebSocketClientMessage::Surrender => self
                .surrender()
                .await
                .unwrap_or_else(|refusal| error_message(Action::Surrender, refusal.into())),
        }
    }

    /// An entry already fired answers its stored shot, and tells no other socket.
    async fn fire(&self, entry: u8) -> Result<WebSocketServerMessage, GameError> {
        let game = self.game().await?;
        let (shot, firing) =
            game_service::fire(&self.state.pool, &game, game_service::entry(entry)?).await?;
        let replayed = firing == Firing::Replayed;
        let message = WebSocketServerMessage::Shot {
            shot: shot.into(),
            replayed,
        };
        if !replayed {
            self.publish(message.clone());
        }
        Ok(message)
    }

    async fn propose(&self, layout: &Layout) -> Result<WebSocketServerMessage, GameError> {
        let game = self.game().await?;
        let verdict = game_service::propose(&self.state.pool, &game, layout).await?;
        if let Verdict::Correct(solved) = &verdict {
            self.publish_end(solved).await;
        }
        Ok(WebSocketServerMessage::Verdict(verdict.into()))
    }

    async fn surrender(&self) -> Result<WebSocketServerMessage, GameError> {
        let game = self.game().await?;
        let surrendered = game_service::surrender(&self.state.pool, &game).await?;
        let shots = game_service::shots_of(&self.state.pool, &surrendered).await?;
        let message = WebSocketServerMessage::Game {
            game: GameView::new(&surrendered, shots),
        };
        self.publish(message.clone());
        Ok(message)
    }

    /// Re-read on every message: another socket may have ended the game meanwhile.
    async fn game(&self) -> Result<Game, GameError> {
        game_service::get_owned(&self.state.pool, self.game_id, self.user_id).await
    }

    async fn snapshot(&self) -> Result<GameView, GameError> {
        let game = self.game().await?;
        let shots = game_service::shots_of(&self.state.pool, &game).await?;
        Ok(GameView::new(&game, shots))
    }

    /// The proposition is stored whatever happens here: a failure only leaves other tabs behind.
    async fn publish_end(&self, solved: &Game) {
        match game_service::shots_of(&self.state.pool, solved).await {
            Ok(shots) => self.publish(WebSocketServerMessage::Game {
                game: GameView::new(solved, shots),
            }),
            Err(error) => {
                tracing::warn!(
                    % error, "a solved game was not pushed to its other sockets"
                );
            }
        }
    }

    fn publish(&self, message: WebSocketServerMessage) {
        self.state.websocket_hub.publish(
            self.game_id,
            WebSocketPush {
                origin: self.id,
                message,
            },
        );
    }
}

/// The next text frame; `None` once the client is gone. axum answers pings itself.
async fn next_text_frame(socket: &mut WebSocket) -> Option<Utf8Bytes> {
    loop {
        match socket.recv().await? {
            Ok(Message::Text(text)) => return Some(text),
            Ok(Message::Close(_)) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

async fn send_message(
    socket: &mut WebSocket,
    message: &WebSocketServerMessage,
) -> Result<(), axum::Error> {
    let text = serde_json::to_string(message).map_err(axum::Error::new)?;
    socket.send(Message::text(text)).await
}

fn error_message(action: Action, refusal: ApiError) -> WebSocketServerMessage {
    let retry_after_ms = refusal.retry_after_ms();
    let (status, detail) = refusal.into_parts();
    WebSocketServerMessage::Error {
        action,
        status: status.as_u16(),
        detail,
        retry_after_ms,
    }
}

/// Say why, then close with 4000 + the status; the client may be gone already, hence no check.
async fn close_with_error(mut socket: WebSocket, refusal: ApiError) {
    let (status, detail) = refusal.into_parts();
    let status = status.as_u16();
    let message = WebSocketServerMessage::Error {
        action: Action::Connect,
        status,
        detail,
        retry_after_ms: None,
    };
    let _ = send_message(&mut socket, &message).await;
    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code: 4000 + status,
            reason: Utf8Bytes::default(),
        })))
        .await;
}

/// The token's `exp` as a deadline for the socket, capped so the addition cannot overflow.
fn token_deadline(exp: i64) -> Instant {
    const CAP: Duration = Duration::from_hours(24 * 365);
    let left = u64::try_from(exp.saturating_sub(Utc::now().timestamp())).unwrap_or(0);
    Instant::now() + Duration::from_secs(left).min(CAP)
}
