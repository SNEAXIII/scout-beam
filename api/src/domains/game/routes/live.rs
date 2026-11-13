//! `/games/{game_id}/ws` — playing a game over one socket: shots, propositions and surrender,
//! with what the player's other sockets on it change pushed in.

use std::time::Duration;

use axum::Router;
use axum::extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::get;
use chrono::Utc;
use tokio::sync::broadcast::{self, error::RecvError};
use tokio::time::{Instant, sleep_until};
use uuid::Uuid;

use crate::auth::AuthError;
use crate::domains::game::Game;
use crate::domains::game::dto::{Action, ClientMessage, GameView, ServerMessage};
use crate::domains::game::error::GameError;
use crate::domains::game::hub::Push;
use crate::domains::game::service::{self, Firing, Verdict};
use crate::domains::user::CurrentUser;
use crate::engine::Layout;
use crate::shared::error::ApiError;
use crate::state::AppState;

/// How long a fresh socket has to send its token.
const AUTH_DEADLINE: Duration = Duration::from_secs(10);

pub fn router() -> Router<AppState> {
    Router::new().route("/{game_id}/ws", get(open_socket))
}

/// `GET /games/{game_id}/ws` — the game's socket.
#[utoipa::path(
    get,
    path = "/games/{game_id}/ws",
    tag = "Games",
    params(("game_id" = Uuid, Path, description = "Game")),
    description = "A browser cannot send a header on a WebSocket: the first message is \
        `{\"type\":\"auth\",\"token\":\"<access token>\"}`, answered by `game`. Then `fire {entry}`, \
        `propose {pieces}` and `surrender` are each answered, in order, by `shot`, `verdict`, `game` \
        or `error` carrying the HTTP status. The player's other sockets on the game are pushed `shot` \
        and `game`. A refused connection, or the token expiring, sends an `error` with action \
        `connect` and closes with code 4000 + status.",
    responses(
        (status = 101, description = "Switched to the game's WebSocket"),
    ),
)]
pub(crate) async fn open_socket(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
    upgrade: WebSocketUpgrade,
) -> Response {
    upgrade.on_upgrade(move |socket| play(socket, state, game_id))
}

async fn play(mut socket: WebSocket, state: AppState, game_id: Uuid) {
    let (session, mut pushes, expires_at) = match join(&mut socket, state, game_id).await {
        Ok(joined) => joined,
        Err(refusal) => return refuse(socket, refusal).await,
    };
    let ended = session.serve(&mut socket, &mut pushes, expires_at).await;
    drop(pushes);
    session.state.hub.release(game_id);
    if let Some(refusal) = ended {
        refuse(socket, refusal).await;
    }
}

/// Authenticate the first message, check the game is the caller's, then send its snapshot.
async fn join(
    socket: &mut WebSocket,
    state: AppState,
    game_id: Uuid,
) -> Result<(Session, broadcast::Receiver<Push>, Instant), ApiError> {
    let claims = state.jwt.decode_access(&first_token(socket).await?)?;
    let CurrentUser(user) = CurrentUser::of(&state, claims.sub).await?;
    service::get_owned(&state.pool, game_id, user.id).await?;

    // Subscribed before the snapshot is read, so no event falls between the two.
    let pushes = state.hub.subscribe(game_id);
    let session = Session {
        state,
        game_id,
        user_id: user.id,
        id: Uuid::new_v4(),
    };
    match session.snapshot().await {
        Ok(game) => {
            // A client already gone is seen by `serve` on its first read.
            let _ = send(socket, &ServerMessage::Game { game }).await;
            Ok((session, pushes, expiry(claims.exp)))
        }
        Err(error) => {
            drop(pushes);
            session.state.hub.release(game_id);
            Err(error.into())
        }
    }
}

async fn first_token(socket: &mut WebSocket) -> Result<String, AuthError> {
    let first = tokio::time::timeout(AUTH_DEADLINE, next_text(socket))
        .await
        .ok()
        .flatten();
    match first.and_then(|text| serde_json::from_str::<ClientMessage>(&text).ok()) {
        Some(ClientMessage::Auth { token }) => Ok(token),
        _ => Err(AuthError::MissingToken),
    }
}

/// One authenticated socket on one game.
struct Session {
    state: AppState,
    game_id: Uuid,
    user_id: Uuid,
    /// Tells this socket's own events apart from the other sockets'.
    id: Uuid,
}

impl Session {
    /// Until the client leaves; `Some` when the server ends the connection.
    async fn serve(
        &self,
        socket: &mut WebSocket,
        pushes: &mut broadcast::Receiver<Push>,
        expires_at: Instant,
    ) -> Option<ApiError> {
        let expiry = sleep_until(expires_at);
        tokio::pin!(expiry);
        loop {
            let outgoing = tokio::select! {
                text = next_text(socket) => self.answer(&text?).await,
                event = pushes.recv() => match event {
                    Ok(push) if push.origin == self.id => continue,
                    Ok(push) => push.message,
                    Err(RecvError::Lagged(_)) => match self.snapshot().await {
                        Ok(game) => ServerMessage::Game { game },
                        Err(error) => return Some(error.into()),
                    },
                    Err(RecvError::Closed) => return None,
                },
                () = &mut expiry => return Some(AuthError::ExpiredToken.into()),
            };
            send(socket, &outgoing).await.ok()?;
        }
    }

    async fn answer(&self, text: &str) -> ServerMessage {
        let message = match serde_json::from_str::<ClientMessage>(text) {
            Ok(message) => message,
            Err(cause) => {
                return error(
                    Action::Message,
                    ApiError::UnprocessableBody(cause.to_string()),
                );
            }
        };
        match message {
            ClientMessage::Auth { .. } => error(
                Action::Message,
                ApiError::BadRequest("Cette connexion est déjà authentifiée".into()),
            ),
            ClientMessage::Fire { entry } => self
                .fire(entry)
                .await
                .unwrap_or_else(|refusal| error(Action::Fire, refusal.into())),
            ClientMessage::Propose { pieces } => self
                .propose(&Layout { pieces })
                .await
                .unwrap_or_else(|refusal| error(Action::Propose, refusal.into())),
            ClientMessage::Surrender => self
                .surrender()
                .await
                .unwrap_or_else(|refusal| error(Action::Surrender, refusal.into())),
        }
    }

    /// An entry already fired answers its stored shot, and tells no other socket.
    async fn fire(&self, entry: u8) -> Result<ServerMessage, GameError> {
        let game = self.game().await?;
        let (shot, firing) = service::fire(&self.state.pool, &game, service::entry(entry)?).await?;
        let replayed = firing == Firing::Replayed;
        let message = ServerMessage::Shot {
            shot: shot.into(),
            replayed,
        };
        if !replayed {
            self.publish(message.clone());
        }
        Ok(message)
    }

    async fn propose(&self, layout: &Layout) -> Result<ServerMessage, GameError> {
        let game = self.game().await?;
        let verdict = service::propose(&self.state.pool, &game, layout).await?;
        if let Verdict::Correct(solved) = &verdict {
            self.publish_end(solved).await;
        }
        Ok(ServerMessage::Verdict(verdict.into()))
    }

    async fn surrender(&self) -> Result<ServerMessage, GameError> {
        let game = self.game().await?;
        let surrendered = service::surrender(&self.state.pool, &game).await?;
        let shots = service::shots_of(&self.state.pool, &surrendered).await?;
        let message = ServerMessage::Game {
            game: GameView::new(&surrendered, shots),
        };
        self.publish(message.clone());
        Ok(message)
    }

    /// Re-read on every message: another socket may have ended the game meanwhile.
    async fn game(&self) -> Result<Game, GameError> {
        service::get_owned(&self.state.pool, self.game_id, self.user_id).await
    }

    async fn snapshot(&self) -> Result<GameView, GameError> {
        let game = self.game().await?;
        let shots = service::shots_of(&self.state.pool, &game).await?;
        Ok(GameView::new(&game, shots))
    }

    /// The proposition is stored whatever happens here: a failure only leaves other tabs behind.
    async fn publish_end(&self, solved: &Game) {
        match service::shots_of(&self.state.pool, solved).await {
            Ok(shots) => self.publish(ServerMessage::Game {
                game: GameView::new(solved, shots),
            }),
            Err(error) => {
                tracing::warn!(%error, "a solved game was not pushed to its other sockets");
            }
        }
    }

    fn publish(&self, message: ServerMessage) {
        self.state.hub.publish(
            self.game_id,
            Push {
                origin: self.id,
                message,
            },
        );
    }
}

/// The next text frame; `None` once the client is gone. axum answers pings itself.
async fn next_text(socket: &mut WebSocket) -> Option<Utf8Bytes> {
    loop {
        match socket.recv().await? {
            Ok(Message::Text(text)) => return Some(text),
            Ok(Message::Close(_)) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

async fn send(socket: &mut WebSocket, message: &ServerMessage) -> Result<(), axum::Error> {
    let text = serde_json::to_string(message).map_err(axum::Error::new)?;
    socket.send(Message::text(text)).await
}

fn error(action: Action, refusal: ApiError) -> ServerMessage {
    let (status, detail) = refusal.into_parts();
    ServerMessage::Error {
        action,
        status: status.as_u16(),
        detail,
    }
}

/// Say why, then close with 4000 + the status; the client may be gone already, hence no check.
async fn refuse(mut socket: WebSocket, refusal: ApiError) {
    let (status, detail) = refusal.into_parts();
    let status = status.as_u16();
    let message = ServerMessage::Error {
        action: Action::Connect,
        status,
        detail,
    };
    let _ = send(&mut socket, &message).await;
    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code: 4000 + status,
            reason: Utf8Bytes::default(),
        })))
        .await;
}

/// The token's `exp` as a deadline for the socket, capped so the addition cannot overflow.
fn expiry(exp: i64) -> Instant {
    const CAP: Duration = Duration::from_hours(24 * 365);
    let left = u64::try_from(exp.saturating_sub(Utc::now().timestamp())).unwrap_or(0);
    Instant::now() + Duration::from_secs(left).min(CAP)
}
