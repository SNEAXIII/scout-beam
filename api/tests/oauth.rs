#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Discord and Google sign-in against a mock provider served on a local port.
//!
//! A code reads `id:email:verified`; the mock hands it back as the access token, then as the profile.

mod common;

use std::collections::HashMap;

use axum::extract::Path;
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use common::{Response, TestApp};
use http::StatusCode;
use scout_beam::Config;
use scout_beam::config::{EmailPepper, OAuthProvider, Secret};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

const EMAIL: &str = "player@example.com";

/// Serve the mock provider and answer its base URL.
async fn mock_provider() -> String {
    let routes = Router::new()
        .route("/{provider}/token", post(token))
        .route("/discord/users/@me", get(discord_profile))
        .route("/google/userinfo", get(google_profile));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, routes).await });
    base
}

/// Refuses the code `refused`, a wrong secret and a redirect URI the API should not have built.
async fn token(
    Path(provider): Path<String>,
    Form(form): Form<HashMap<String, String>>,
) -> (StatusCode, Json<Value>) {
    let expected_redirect = format!("http://localhost:3001/login/callback/{provider}");
    let valid = form.get("client_secret").map(String::as_str) == Some("client-secret")
        && form.get("grant_type").map(String::as_str) == Some("authorization_code")
        && form.get("redirect_uri") == Some(&expected_redirect)
        && form.get("code").is_some_and(|code| code != "refused");

    if valid {
        let code = &form["code"];
        (
            StatusCode::OK,
            Json(json!({ "access_token": code, "token_type": "Bearer" })),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_grant" })),
        )
    }
}

fn profile(headers: &HeaderMap) -> (String, String, bool) {
    let token = headers["authorization"]
        .to_str()
        .unwrap()
        .strip_prefix("Bearer ")
        .unwrap();
    let mut parts = token.split(':');
    let id = parts.next().unwrap().to_owned();
    let email = parts.next().unwrap().to_owned();
    let verified = parts.next() == Some("true");
    (id, email, verified)
}

async fn discord_profile(headers: HeaderMap) -> Json<Value> {
    let (id, email, verified) = profile(&headers);
    Json(json!({ "id": id, "username": "ignored", "email": email, "verified": verified }))
}

async fn google_profile(headers: HeaderMap) -> Json<Value> {
    let (sub, email, email_verified) = profile(&headers);
    Json(json!({ "sub": sub, "email": email, "email_verified": email_verified }))
}

fn provider(base: &str, name: &str, profile_path: &str) -> OAuthProvider {
    OAuthProvider {
        client_id: format!("{name}-client"),
        client_secret: Secret::new("client-secret"),
        authorize_url: format!("{base}/{name}/authorize"),
        token_url: format!("{base}/{name}/token"),
        profile_url: format!("{base}{profile_path}"),
    }
}

fn config(base: &str, pepper: &str, version: i16) -> Config {
    Config {
        discord: Some(provider(base, "discord", "/discord/users/@me")),
        google: Some(provider(base, "google", "/google/userinfo")),
        email_pepper: Some(EmailPepper {
            secret: Secret::new(pepper),
            version,
        }),
        ..Config::for_tests()
    }
}

async fn app(pool: PgPool) -> TestApp {
    let base = mock_provider().await;
    TestApp::with_config(pool, config(&base, "pepper-one", 1))
}

impl TestApp {
    async fn sign_in(&self, provider: &str, code: &str, token: Option<&str>) -> Response {
        self.post(&format!("/auth/{provider}"), token, json!({ "code": code }))
            .await
    }

    async fn user_count(&self) -> i64 {
        sqlx::query_scalar(r#"SELECT count(*) FROM "user""#)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn email_hash_of(&self, discord_id: &str) -> (Option<String>, Option<i16>) {
        sqlx::query_as(r#"SELECT email_hash, email_hash_version FROM "user" WHERE discord_id = $1"#)
            .bind(discord_id)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }
}

fn verified(id: &str) -> String {
    format!("{id}:{EMAIL}:true")
}

#[sqlx::test]
async fn a_first_sign_in_creates_an_account_with_a_generated_login_and_no_plaintext_email(
    pool: PgPool,
) {
    let app = app(pool).await;

    let signed = app.sign_in("discord", &verified("d1"), None).await;

    signed.assert_status(StatusCode::OK);
    let token = signed.body["access_token"].as_str().unwrap();
    let session = app.get("/auth/session", Some(token)).await;
    session.assert_status(StatusCode::OK);
    let login = session.body["login"].as_str().unwrap();
    assert!(
        !login.contains("player") && !login.contains("ignored"),
        "{login}"
    );

    let row: String = sqlx::query_scalar(r#"SELECT row_to_json(u)::text FROM "user" u"#)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert!(!row.to_lowercase().contains("example.com"), "{row}");
    assert_eq!(app.email_hash_of("d1").await.1, Some(1));
}

#[sqlx::test]
async fn a_second_sign_in_returns_the_same_account(pool: PgPool) {
    let app = app(pool).await;

    let first = app.sign_in("discord", &verified("d1"), None).await;
    let second = app.sign_in("discord", &verified("d1"), None).await;

    second.assert_status(StatusCode::OK);
    assert_eq!(app.user_count().await, 1);
    let sub = |response: &Response| {
        app.jwt
            .decode_access(response.body["access_token"].as_str().unwrap())
            .unwrap()
            .sub
    };
    assert_eq!(sub(&first), sub(&second));
}

#[sqlx::test]
async fn two_racing_first_sign_ins_end_on_one_account(pool: PgPool) {
    let app = app(pool).await;
    let code = verified("d1");

    let (first, second) = tokio::join!(
        app.sign_in("discord", &code, None),
        app.sign_in("discord", &code, None)
    );

    first.assert_status(StatusCode::OK);
    second.assert_status(StatusCode::OK);
    assert_eq!(app.user_count().await, 1);
}

#[sqlx::test]
async fn discord_then_google_with_the_same_verified_email_give_one_account(pool: PgPool) {
    let app = app(pool).await;

    app.sign_in("discord", &verified("d1"), None)
        .await
        .assert_status(StatusCode::OK);
    app.sign_in("google", &verified("g1"), None)
        .await
        .assert_status(StatusCode::OK);

    let (discord_id, google_id): (Option<String>, Option<String>) =
        sqlx::query_as(r#"SELECT discord_id, google_id FROM "user""#)
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!(
        (discord_id.as_deref(), google_id.as_deref()),
        (Some("d1"), Some("g1"))
    );
}

#[sqlx::test]
async fn an_unverified_email_creates_a_separate_account_without_a_hash(pool: PgPool) {
    let app = app(pool).await;

    app.sign_in("discord", &verified("d1"), None).await;
    app.sign_in("google", &format!("g1:{EMAIL}:false"), None)
        .await
        .assert_status(StatusCode::OK);

    assert_eq!(app.user_count().await, 2);
    let hash: Option<String> =
        sqlx::query_scalar(r#"SELECT email_hash FROM "user" WHERE google_id = 'g1'"#)
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert!(hash.is_none(), "an unverified email must never be hashed");
}

#[sqlx::test]
async fn a_stale_email_hash_is_rehashed_with_the_current_pepper(pool: PgPool) {
    let base = mock_provider().await;
    let before = TestApp::with_config(pool.clone(), config(&base, "pepper-one", 1));
    before
        .sign_in("discord", &verified("d1"), None)
        .await
        .assert_status(StatusCode::OK);
    let (old_hash, _) = before.email_hash_of("d1").await;

    let after = TestApp::with_config(pool, config(&base, "pepper-two", 2));
    after
        .sign_in("discord", &verified("d1"), None)
        .await
        .assert_status(StatusCode::OK);

    let (new_hash, version) = after.email_hash_of("d1").await;
    assert_eq!(version, Some(2));
    assert!(new_hash.is_some() && new_hash != old_hash);
}

#[sqlx::test]
async fn a_second_discord_id_on_a_linked_account_is_a_conflict(pool: PgPool) {
    let app = app(pool).await;
    app.sign_in("discord", &verified("d1"), None).await;

    let second = app.sign_in("discord", &verified("d2"), None).await;

    second.assert_status(StatusCode::CONFLICT);
    assert!(second.detail().contains("déjà lié"), "{}", second.detail());
    assert_eq!(app.user_count().await, 1);
}

#[sqlx::test]
async fn a_guest_bearer_is_converted_and_keeps_its_login(pool: PgPool) {
    let app = app(pool).await;
    let (guest_id, guest_token) = app.guest().await;
    let login: String = sqlx::query_scalar(r#"SELECT login FROM "user" WHERE id = $1"#)
        .bind(guest_id)
        .fetch_one(&app.pool)
        .await
        .unwrap();

    app.sign_in("google", &verified("g1"), Some(&guest_token))
        .await
        .assert_status(StatusCode::OK);

    let (id, kept_login, guest, google_id): (Uuid, String, bool, Option<String>) =
        sqlx::query_as(r#"SELECT id, login, guest, google_id FROM "user""#)
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!((id, kept_login, guest), (guest_id, login, false));
    assert_eq!(google_id.as_deref(), Some("g1"));
}

#[sqlx::test]
async fn a_refused_code_answers_a_generic_401_and_creates_nothing(pool: PgPool) {
    let app = app(pool).await;

    let refused = app.sign_in("discord", "refused", None).await;

    refused.assert_status(StatusCode::UNAUTHORIZED);
    assert!(
        !refused.detail().contains("invalid_grant"),
        "{}",
        refused.detail()
    );
    assert_eq!(app.user_count().await, 0);
}

#[sqlx::test]
async fn only_configured_providers_are_listed(pool: PgPool) {
    let base = mock_provider().await;
    let app = TestApp::with_config(
        pool,
        Config {
            google: None,
            ..config(&base, "pepper-one", 1)
        },
    );

    let listed = app.get("/auth/providers", None).await;

    listed.assert_status(StatusCode::OK);
    assert_eq!(listed.body["dev_login"], true);
    let providers = listed.body["providers"].as_array().unwrap();
    assert_eq!(providers.len(), 1, "{providers:?}");
    assert_eq!(providers[0]["name"], "discord");
    let url = providers[0]["authorize_url"].as_str().unwrap();
    assert!(
        url.starts_with(&format!("{base}/discord/authorize?")),
        "{url}"
    );
    for part in [
        "client_id=discord-client",
        "redirect_uri=http%3A%2F%2Flocalhost%3A3001%2Flogin%2Fcallback%2Fdiscord",
        "response_type=code",
        "scope=identify+email",
    ] {
        assert!(url.contains(part), "{part} missing from {url}");
    }
    assert!(!url.contains("state="), "{url}");
}

#[sqlx::test]
async fn without_configuration_no_provider_is_listed_nor_usable(pool: PgPool) {
    let app = TestApp::new(pool);

    let listed = app.get("/auth/providers", None).await;
    assert_eq!(listed.body["providers"], json!([]));

    app.sign_in("discord", &verified("d1"), None)
        .await
        .assert_status(StatusCode::NOT_FOUND);
}
