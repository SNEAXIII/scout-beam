#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// Each test binary compiles this module on its own and uses only part of it.
#![allow(dead_code)]

//! Test harness: the real router, a throwaway database, no socket.
//!
//! `#[sqlx::test]` hands each test its own migrated database; the router is called through `oneshot`.

use axum::Router;
use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use scout_beam::auth::JwtCodec;
use scout_beam::engine::Layout;
use scout_beam::shared::role::Role;
use scout_beam::{AppState, Config, build_app};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

pub struct TestApp {
    router: Router,
    pub pool: PgPool,
    pub jwt: JwtCodec,
}

impl TestApp {
    pub fn new(pool: PgPool) -> Self {
        Self::with_config(pool, Config::for_tests())
    }

    pub fn with_config(pool: PgPool, config: Config) -> Self {
        let state = AppState::new(pool.clone(), config);
        let jwt = state.jwt.clone();

        Self {
            router: build_app(state).expect("router builds"),
            pool,
            jwt,
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

        Response { status, body }
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
        let body = serde_json::to_value(layout).expect("the layout encodes");
        self.post(&format!("/games/{game_id}/propositions"), token, body)
            .await
    }

    /// Solve a game the way a player does: propose its hidden layout.
    pub async fn solve(&self, game_id: Uuid, token: &str) {
        let layout = self.stored_layout(game_id).await;
        let proposed = self.propose(game_id, Some(token), &layout).await;
        proposed.assert_status(StatusCode::CREATED);
        assert_eq!(proposed.body["correct"], true, "{}", proposed.body);
    }

    /// Seed an account and return it with a matching token.
    pub async fn seed_with_token(&self, login: &str, role: Role) -> (Uuid, String) {
        let id = self.seed(login, role).await;
        let token = self.token_for(id, role);
        (id, token)
    }
}

pub struct Response {
    pub status: StatusCode,
    pub body: Value,
}

impl Response {
    /// The `detail` field of an error payload, as text.
    pub fn detail(&self) -> String {
        self.body
            .get("detail")
            .map(ToString::to_string)
            .unwrap_or_default()
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
    let mut builder = Request::builder().method(method).uri(uri);

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
