//! `/games/{game_id}/ws` — playing a game over one socket: shots, propositions and surrender,
//! with what the player's other sockets on it, and in a room the opponent, change pushed in.

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
use crate::dto::game_dto::{
    Action, GameView, PropositionResponse, WebSocketClientMessage, WebSocketServerMessage,
};
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
        player's other sockets on the game are pushed `shot` and `game`. In a room, `game`, `shot` \
        and `verdict` also carry `elapsed_ms`, since the game started, and `proposition_in_ms`, \
        before the next proposition (0: now), both by the server's clock: 15 shots at the start, \
        the k-th earned one at 5 s × k + 0.25 s × k × (k − 1), so the shots left are 15 + earned − \
        shots fired. A new shot with none left, or a proposition within its cooldown, is an `error` \
        409 with `retry_after_ms`. Both players' sockets are pushed `opponent {shots}`, the \
        opponent's shot count, after each of their new shots, and each its own `game` when the \
        room ends. A refused connection, or the token expiring, sends an `error` with action `connect` and \
        closes with code 4000 + status.",
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
    check_origin(&state, &headers)?;
    // An auth refusal still upgrades: a browser reads the 4401 close code, never a 401 status.
    Ok(upgrade.on_upgrade(move |socket| run_websocket(socket, state, game_id, caller)))
}

/// A browser sends the cookies whichever page opens the socket: only the front may.
/// No `Origin` at all is a tool or a test, which can send any cookie it likes anyway.
pub(crate) fn check_origin(state: &AppState, headers: &HeaderMap) -> Result<(), ApiError> {
    match headers.get(ORIGIN) {
        Some(origin) if origin.as_bytes() != state.config.cors_allowed_origin.as_bytes() => {
            Err(ApiError::Forbidden("Origine non autorisée".into()))
        }
        _ => Ok(()),
    }
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
    session.release(pushes);
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
) -> Result<(WebSocketSession, Pushes, Instant), ApiError> {
    let CurrentUser(user) = CurrentUser::of(&state, identity.user_id).await?;
    let game = game_service::get_owned(&state.pool, game_id, user.id).await?;

    // Subscribed before the snapshot is read, so no event falls between the two.
    let pushes = Pushes {
        game: state.websocket_hub.subscribe(game_id),
        room: game
            .room_id
            .map(|room_id| state.websocket_hub.subscribe(room_id)),
    };
    let session = WebSocketSession {
        state,
        game_id,
        room_id: game.room_id,
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
            session.release(pushes);
            Err(error.into())
        }
    }
}

struct Pushes {
    game: broadcast::Receiver<WebSocketPush>,
    room: Option<broadcast::Receiver<WebSocketPush>>,
}

/// One authenticated socket on one game.
struct WebSocketSession {
    state: AppState,
    game_id: Uuid,
    room_id: Option<Uuid>,
    user_id: Uuid,
    /// Tells this socket's own events apart from the other sockets'.
    id: Uuid,
}

impl WebSocketSession {
    /// Until the client leaves; `Some` when the server ends the connection.
    async fn serve(
        &self,
        socket: &mut WebSocket,
        pushes: &mut Pushes,
        expires_at: Instant,
    ) -> Option<ApiError> {
        let expiry = sleep_until(expires_at);
        tokio::pin!(expiry);
        loop {
            let outgoing = tokio::select! {
                text = next_text_frame(socket) => Ok(Some(self.answer_client_message(&text?).await)),
                event = pushes.game.recv() => self.pushed(event).await,
                event = next_room_push(&mut pushes.room) => self.pushed(event).await,
                () = &mut expiry => Err(Some(AuthError::ExpiredToken.into())),
            };
            match outgoing {
                Ok(Some(message)) => send_message(socket, &message).await.ok()?,
                Ok(None) => {}
                Err(ended) => return ended,
            }
        }
    }

    /// What a push sends this socket: `None` to skip it, `Err` to end the connection.
    async fn pushed(
        &self,
        event: Result<WebSocketPush, RecvError>,
    ) -> Result<Option<WebSocketServerMessage>, Option<ApiError>> {
        match event {
            Ok(push) if push.origin == Some(self.id) || push.origin == Some(self.user_id) => {
                Ok(None)
            }
            Ok(WebSocketPush {
                message: Some(message),
                ..
            }) => Ok(Some(message)),
            Ok(_) | Err(RecvError::Lagged(_)) => match self.snapshot().await {
                Ok(game) => Ok(Some(WebSocketServerMessage::Game { game })),
                Err(error) => Err(Some(error.into())),
            },
            Err(RecvError::Closed) => Err(None),
        }
    }

    /// Once the socket closes: the receivers go before their channels are released.
    fn release(&self, pushes: Pushes) {
        drop(pushes);
        let hub = &self.state.websocket_hub;
        hub.release(self.game_id);
        if let Some(room_id) = self.room_id {
            hub.release(room_id);
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
        let (shot, firing, pace) =
            game_service::fire(&self.state.pool, &game, game_service::entry(entry)?).await?;
        let message = WebSocketServerMessage::Shot {
            shot: shot.into(),
            replayed: firing == Firing::Replayed,
            pace: pace.map(Into::into),
        };
        if firing == Firing::New {
            self.publish(message.clone());
            if let Some(pace) = pace {
                let opponent = WebSocketServerMessage::Opponent { shots: pace.fired };
                self.publish_to_room(Some(self.user_id), Some(opponent));
            }
        }
        Ok(message)
    }

    async fn propose(&self, layout: &Layout) -> Result<WebSocketServerMessage, GameError> {
        let game = self.game().await?;
        let (verdict, pace) = game_service::propose(&self.state.pool, &game, layout).await?;
        if let Verdict::Correct(solved) = &verdict {
            self.publish_end(solved).await;
        }
        let answer = PropositionResponse::new(&verdict, pace);
        Ok(WebSocketServerMessage::Verdict(answer))
    }

    async fn surrender(&self) -> Result<WebSocketServerMessage, GameError> {
        let game = self.game().await?;
        let surrendered = game_service::surrender(&self.state.pool, &game).await?;
        let shots = game_service::shots_of(&self.state.pool, &surrendered).await?;
        let message = WebSocketServerMessage::Game {
            game: GameView::new(&surrendered, shots),
        };
        self.publish_end_game(message.clone());
        Ok(message)
    }

    /// Re-read on every message: another socket may have ended the game meanwhile.
    async fn game(&self) -> Result<Game, GameError> {
        game_service::get_owned(&self.state.pool, self.game_id, self.user_id).await
    }

    async fn snapshot(&self) -> Result<GameView, GameError> {
        let game = self.game().await?;
        let shots = game_service::shots_of(&self.state.pool, &game).await?;
        let pace = game_service::pace_of(&self.state.pool, &game).await?;
        Ok(GameView::new(&game, shots).with_pace(pace))
    }

    /// The proposition is stored whatever happens here: a failure only leaves other tabs behind.
    async fn publish_end(&self, solved: &Game) {
        match game_service::shots_of(&self.state.pool, solved).await {
            Ok(shots) => self.publish_end_game(WebSocketServerMessage::Game {
                game: GameView::new(solved, shots),
            }),
            Err(error) => {
                tracing::warn!(
                    % error, "a solved game was not pushed to its other sockets"
                );
            }
        }
    }

    /// In a room the end is the room's: every socket of both players sends its own game.
    fn publish_end_game(&self, message: WebSocketServerMessage) {
        if self.room_id.is_some() {
            self.publish_to_room(Some(self.id), None);
        } else {
            self.publish(message);
        }
    }

    fn publish(&self, message: WebSocketServerMessage) {
        self.state.websocket_hub.publish(
            self.game_id,
            WebSocketPush {
                origin: Some(self.id),
                message: Some(message),
            },
        );
    }

    fn publish_to_room(&self, origin: Option<Uuid>, message: Option<WebSocketServerMessage>) {
        if let Some(room_id) = self.room_id {
            let push = WebSocketPush { origin, message };
            self.state.websocket_hub.publish(room_id, push);
        }
    }
}

async fn next_room_push(
    room: &mut Option<broadcast::Receiver<WebSocketPush>>,
) -> Result<WebSocketPush, RecvError> {
    match room {
        Some(pushes) => pushes.recv().await,
        None => std::future::pending().await,
    }
}

/// The next text frame; `None` once the client is gone. axum answers pings itself.
pub(crate) async fn next_text_frame(socket: &mut WebSocket) -> Option<Utf8Bytes> {
    loop {
        match socket.recv().await? {
            Ok(Message::Text(text)) => return Some(text),
            Ok(Message::Close(_)) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

pub(crate) async fn send_message(
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
pub(crate) async fn close_with_error(mut socket: WebSocket, refusal: ApiError) {
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
pub(crate) fn token_deadline(exp: i64) -> Instant {
    const CAP: Duration = Duration::from_hours(24 * 365);
    let left = u64::try_from(exp.saturating_sub(Utc::now().timestamp())).unwrap_or(0);
    Instant::now() + Duration::from_secs(left).min(CAP)
}
