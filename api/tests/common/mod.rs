#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// Each test binary compiles this module on its own and uses only part of it.
#![allow(dead_code)]

//! Test harness: the real router, a throwaway database.
//!
//! `#[sqlx::test]` hands each test its own migrated database; HTTP calls go through `oneshot`, game
//! sockets through the same router served on a local port.

use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use futures_util::{SinkExt, StreamExt};
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use scout_beam::auth::JwtCodec;
use scout_beam::engine::Layout;
use scout_beam::shared::role::Role;
use scout_beam::{AppState, Config, build_app};
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tower::ServiceExt;
use uuid::Uuid;

pub struct TestApp {
    router: Router,
    pub pool: PgPool,
    pub jwt: JwtCodec,
    addr: SocketAddr,
}

impl TestApp {
    pub fn new(pool: PgPool) -> Self {
        Self::with_config(pool, Config::for_tests())
    }

    pub fn with_config(pool: PgPool, config: Config) -> Self {
        let state = AppState::new(pool.clone(), config).expect("state builds");
        let jwt = state.jwt.clone();
        let router = build_app(state).expect("router builds");

        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a local port is free");
        listener
            .set_nonblocking(true)
            .expect("the listener is non-blocking");
        let addr = listener.local_addr().expect("the port is known");
        let listener =
            tokio::net::TcpListener::from_std(listener).expect("tokio takes the listener");
        let served = router.clone();
        tokio::spawn(async move { axum::serve(listener, served).await });

        Self {
            router,
            pool,
            jwt,
            addr,
        }
    }

    /// A game's socket: authenticated with `token`, or sent a shot first when there is none.
    /// `Ok` holds the socket and the game it answered; `Err` the refusal, as a response.
    pub async fn join(&self, game_id: Uuid, token: Option<&str>) -> Result<GameSocket, Response> {
        let mut socket = self.join_raw(game_id, token).await;
        let answer = answered(socket.next().await.expect("the server answers"));
        if answer.status == StatusCode::OK {
            socket.joined = answer.body;
            Ok(socket)
        } else {
            Err(answer)
        }
    }

    /// The socket right after the upgrade, its first message still unread. The token goes in a
    /// bearer header: a browser sends the access cookie instead.
    pub async fn join_raw(&self, game_id: Uuid, token: Option<&str>) -> GameSocket {
        let headers = token
            .map(|token| (http::header::AUTHORIZATION, format!("Bearer {token}")))
            .into_iter();
        self.join_with(game_id, headers)
            .await
            .expect("the socket opens")
    }

    /// The socket opened with these headers, or the HTTP status that refused the upgrade.
    pub async fn join_with(
        &self,
        game_id: Uuid,
        headers: impl IntoIterator<Item = (http::HeaderName, String)>,
    ) -> Result<GameSocket, StatusCode> {
        let mut request = format!("ws://{}/games/{game_id}/ws", self.addr)
            .into_client_request()
            .expect("the URL is a request");
        for (name, value) in headers {
            request
                .headers_mut()
                .insert(name, value.parse().expect("a valid header"));
        }
        match tokio_tungstenite::connect_async(request).await {
            Ok((stream, _)) => Ok(GameSocket {
                stream,
                joined: Value::Null,
            }),
            Err(tungstenite::Error::Http(response)) => Err(response.status()),
            Err(other) => panic!("the socket did not open: {other}"),
        }
    }

    /// One message on a fresh socket, answered as an HTTP endpoint would have.
    pub async fn play(&self, game_id: Uuid, token: Option<&str>, message: Value) -> Response {
        match self.join(game_id, token).await {
            Ok(mut socket) => socket.request(message).await,
            Err(refused) => refused,
        }
    }

    pub async fn send(&self, request: Request<Body>) -> Response {
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("router answers");

        let status = response.status();
        let cookies = response
            .headers()
            .get_all(http::header::SET_COOKIE)
            .iter()
            .map(|value| value.to_str().expect("a readable cookie").to_owned())
            .collect();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body collects")
            .to_bytes();

        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        };

        Response {
            status,
            body,
            cookies,
        }
    }

    /// A request carrying only a `Cookie` header, as the browser sends it.
    pub async fn with_cookie(&self, method: http::Method, uri: &str, cookie: &str) -> Response {
        let mut request = build(method, uri, None, None);
        request.headers_mut().insert(
            http::header::COOKIE,
            cookie.parse().expect("a valid cookie header"),
        );
        self.send(request).await
    }

    pub async fn get(&self, uri: &str, token: Option<&str>) -> Response {
        self.send(build(http::Method::GET, uri, token, None)).await
    }

    pub async fn post(&self, uri: &str, token: Option<&str>, body: Value) -> Response {
        self.send(build(http::Method::POST, uri, token, Some(body)))
            .await
    }

    pub async fn put(&self, uri: &str, token: Option<&str>, body: Option<Value>) -> Response {
        self.send(build(http::Method::PUT, uri, token, body)).await
    }

    pub async fn patch(&self, uri: &str, token: Option<&str>, body: Option<Value>) -> Response {
        self.send(build(http::Method::PATCH, uri, token, body))
            .await
    }

    pub async fn delete(&self, uri: &str, token: Option<&str>, body: Option<Value>) -> Response {
        self.send(build(http::Method::DELETE, uri, token, body))
            .await
    }

    /// An access token for an account, without going through `/auth/dev-login`.
    pub fn token_for(&self, user_id: Uuid, role: Role) -> String {
        self.jwt
            .access_token(user_id, role)
            .expect("token is minted")
    }

    /// Insert an account directly, so a test can start from any state.
    pub async fn seed(&self, login: &str, role: Role) -> Uuid {
        sqlx::query_scalar::<_, Uuid>(
            r#"INSERT INTO "user" (login, role) VALUES ($1, $2) RETURNING id"#,
        )
        .bind(login)
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .expect("user is seeded")
    }

    /// Start a game through the API and return its id.
    pub async fn start_game(&self, token: &str) -> Uuid {
        let created = self.post("/games", Some(token), json!({})).await;
        created.assert_status(StatusCode::CREATED);
        created.body["id"]
            .as_str()
            .expect("the id is returned")
            .parse()
            .expect("the id is a uuid")
    }

    /// The hidden layout, read straight from the table.
    pub async fn stored_layout(&self, game_id: Uuid) -> Layout {
        let layout: Value = sqlx::query_scalar("SELECT layout FROM game WHERE id = $1")
            .bind(game_id)
            .fetch_one(&self.pool)
            .await
            .expect("the game is stored");
        serde_json::from_value(layout).expect("the stored layout is the engine's")
    }

    /// Replace the bot's draw, so a test knows what is hidden.
    pub async fn hide_layout(&self, game_id: Uuid, layout: &Layout) {
        sqlx::query("UPDATE game SET layout = $2 WHERE id = $1")
            .bind(game_id)
            .bind(serde_json::to_value(layout).expect("the layout encodes"))
            .execute(&self.pool)
            .await
            .expect("the layout is hidden");
    }

    pub async fn propose(&self, game_id: Uuid, token: Option<&str>, layout: &Layout) -> Response {
        let mut message = serde_json::to_value(layout).expect("the layout encodes");
        message["type"] = json!("propose");
        self.play(game_id, token, message).await
    }

    /// Solve a game the way a player does: propose its hidden layout.
    pub async fn solve(&self, game_id: Uuid, token: &str) {
        let layout = self.stored_layout(game_id).await;
        let proposed = self.propose(game_id, Some(token), &layout).await;
        proposed.assert_status(StatusCode::CREATED);
        assert_eq!(proposed.body["correct"], true, "{}", proposed.body);
    }

    /// Sign in as a new guest through the API: its id and access token.
    pub async fn guest(&self) -> (Uuid, String) {
        let created = self.post("/auth/guest", None, json!({})).await;
        created.assert_status(StatusCode::CREATED);
        let token = created.access_token();
        let id = self
            .jwt
            .decode_access(&token)
            .expect("the token decodes")
            .sub;
        (id, token)
    }

    /// Seed an account and return it with a matching token.
    pub async fn seed_with_token(&self, login: &str, role: Role) -> (Uuid, String) {
        let id = self.seed(login, role).await;
        let token = self.token_for(id, role);
        (id, token)
    }
}

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub body: Value,
    /// Every `Set-Cookie` header, verbatim.
    pub cookies: Vec<String>,
}

pub struct GameSocket {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    /// The game the server answered on joining.
    pub joined: Value,
}

impl GameSocket {
    pub async fn send(&mut self, message: Value) {
        self.stream
            .send(Message::text(message.to_string()))
            .await
            .expect("the message is sent");
    }

    /// The next server message, or `None` once the server closed; the close code comes along.
    pub async fn next(&mut self) -> Result<Value, Option<u16>> {
        loop {
            let frame = tokio::time::timeout(Duration::from_secs(5), self.stream.next())
                .await
                .expect("the server answers within 5 s");
            match frame {
                Some(Ok(Message::Text(text))) => {
                    return Ok(serde_json::from_str(&text).expect("the server sends JSON"));
                }
                Some(Ok(Message::Close(frame))) => return Err(frame.map(|f| u16::from(f.code))),
                Some(Ok(_)) => {}
                None | Some(Err(_)) => return Err(None),
            }
        }
    }

    /// Send and read the answer, mapped to the status and body the HTTP endpoint had.
    pub async fn request(&mut self, message: Value) -> Response {
        let fired = (message["type"] == "fire").then(|| message["entry"].clone());
        self.send(message).await;
        loop {
            let answer = self
                .next()
                .await
                .expect("the server answers before closing");
            // Another tab's shot pushed to this socket is not this fire's answer.
            let pushed = answer["type"] == "shot"
                && fired
                    .as_ref()
                    .is_some_and(|entry| answer["shot"]["entry"] != *entry);
            if !pushed {
                return answered(answer);
            }
        }
    }

    /// Nothing arrives for a moment: no push is on its way.
    pub async fn assert_quiet(&mut self) {
        let frame = tokio::time::timeout(Duration::from_millis(300), self.stream.next()).await;
        assert!(frame.is_err(), "unexpected message: {frame:?}");
    }
}

/// `game` is a 200, `shot` a 201 or a replayed 200, `verdict` a 201, `error` its own status.
pub fn answered(mut message: Value) -> Response {
    let kind = message["type"]
        .as_str()
        .expect("every message is typed")
        .to_owned();
    let (status, body) = match kind.as_str() {
        "game" => (StatusCode::OK, message["game"].take()),
        "shot" if message["replayed"] == true => (StatusCode::OK, message["shot"].take()),
        "shot" => (StatusCode::CREATED, message["shot"].take()),
        "verdict" => {
            message.as_object_mut().expect("an object").remove("type");
            (StatusCode::CREATED, message)
        }
        "error" => {
            let status = message["status"].as_u64().expect("an error has a status");
            let status = StatusCode::from_u16(u16::try_from(status).unwrap()).unwrap();
            (status, message)
        }
        other => panic!("unknown message type {other}"),
    };
    Response {
        status,
        body,
        cookies: Vec::new(),
    }
}

impl Response {
    /// The full `Set-Cookie` header of one cookie.
    pub fn set_cookie(&self, name: &str) -> Option<&str> {
        let prefix = format!("{name}=");
        self.cookies
            .iter()
            .map(String::as_str)
            .find(|cookie| cookie.starts_with(&prefix))
    }

    /// The value of one cookie the response sets.
    pub fn cookie(&self, name: &str) -> Option<String> {
        let header = self.set_cookie(name)?;
        let pair = header.split(';').next()?;
        Some(pair.split_once('=')?.1.to_owned())
    }

    /// The access token a sign-in set, to use as a bearer.
    pub fn access_token(&self) -> String {
        self.cookie("scout_beam_access")
            .expect("the access cookie is set")
    }

    /// The `detail` field of an error payload, as text.
    pub fn detail(&self) -> String {
        self.body
            .get("detail")
            .map_or_else(String::new, ToString::to_string)
    }

    /// The `code` field of an error payload, the reason the front translates.
    pub fn code(&self) -> &str {
        self.body["code"].as_str().unwrap_or_default()
    }

    pub fn assert_status(&self, expected: StatusCode) -> &Self {
        assert_eq!(
            self.status, expected,
            "unexpected status, body was {}",
            self.body
        );
        self
    }
}

fn build(
    method: http::Method,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> Request<Body> {
    // Rate limiting keys on the client address: `oneshot` has none, so every request names one.
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("x-forwarded-for", "203.0.113.7");

    if let Some(token) = token {
        builder = builder.header(http::header::AUTHORIZATION, format!("Bearer {token}"));
    }

    match body {
        Some(value) => builder
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_vec(&value).expect("body encodes"),
            ))
            .expect("request builds"),
        None => builder.body(Body::empty()).expect("request builds"),
    }
}

/// Convenience for the many `{"login": "..."}` payloads.
pub fn login_body(login: &str) -> Value {
    json!({ "login": login })
}
