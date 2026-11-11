#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `/auth/*` — minting a session, reading it, renewing it.

mod common;

use common::{TestApp, login_body};
use http::StatusCode;
use scout_beam::Config;
use scout_beam::config::Environment;
use scout_beam::domains::user::repository;
use scout_beam::shared::role::Role;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// The state a lost race leaves behind: seeding the row between lookup and insert *is* that state.
#[sqlx::test]
async fn a_login_taken_between_the_lookup_and_the_insert_is_not_an_error(pool: PgPool) {
    let app = TestApp::new(pool);
    app.seed("Sneaxiii", Role::User).await;

    let outcome = repository::insert_if_absent(&app.pool, "Sneaxiii", Role::User).await;

    assert!(
        matches!(outcome, Ok(None)),
        "the conflict must be absorbed so the caller can read the row back, got {outcome:?}"
    );
}

#[sqlx::test]
async fn dev_login_creates_the_account_on_first_sight(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app
        .post("/auth/dev-login", None, login_body("Sneaxiii"))
        .await;

    response.assert_status(StatusCode::OK);
    assert_eq!(response.body["token_type"], "bearer");
    assert!(response.body["access_token"].is_string());
    assert!(response.body["refresh_token"].is_string());
}

#[sqlx::test]
async fn dev_login_is_not_mounted_in_prod(pool: PgPool) {
    let config = Config {
        environment: Environment::Prod,
        ..Config::for_tests()
    };
    let app = TestApp::with_config(pool, config);

    app.post("/auth/dev-login", None, json!({ "role": "super_admin" }))
        .await
        .assert_status(StatusCode::NOT_FOUND);

    let count: i64 = sqlx::query_scalar(r#"SELECT count(*) FROM "user""#)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(count, 0, "no account may be created in prod");
}

#[sqlx::test]
async fn dev_login_reuses_the_account_on_second_sight(pool: PgPool) {
    let app = TestApp::new(pool);

    app.post("/auth/dev-login", None, login_body("Sneaxiii"))
        .await;
    app.post("/auth/dev-login", None, login_body("Sneaxiii"))
        .await
        .assert_status(StatusCode::OK);

    let count: i64 = sqlx::query_scalar(r#"SELECT count(*) FROM "user""#)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(count, 1, "a second login must not create a second account");
}

#[sqlx::test]
async fn dev_login_records_the_login_date(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.seed("Sneaxiii", Role::User).await;

    app.post("/auth/dev-login", None, login_body("Sneaxiii"))
        .await;

    let seen: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar(r#"SELECT last_login_date FROM "user" WHERE id = $1"#)
            .bind(id)
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert!(seen.is_some());
}

#[sqlx::test]
async fn dev_login_rejects_an_invalid_login(pool: PgPool) {
    let app = TestApp::new(pool);

    app.post("/auth/dev-login", None, login_body("no"))
        .await
        .assert_status(StatusCode::UNPROCESSABLE_ENTITY);

    app.post("/auth/dev-login", None, login_body("drop-table"))
        .await
        .assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn session_returns_the_caller_profile(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("Sneaxiii", Role::Admin).await;

    let response = app.get("/auth/session", Some(&token)).await;

    response.assert_status(StatusCode::OK);
    assert_eq!(response.body["login"], "Sneaxiii");
    assert_eq!(response.body["role"], "admin");
    assert!(response.body.get("created_at").is_some());
    assert!(
        response.body.get("id").is_none() && response.body.get("updated_at").is_none(),
        "the profile must not leak internal columns"
    );
}

#[sqlx::test]
async fn session_requires_a_token(pool: PgPool) {
    let app = TestApp::new(pool);

    app.get("/auth/session", None)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);

    app.get("/auth/session", Some("not-a-jwt"))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn session_refuses_a_token_whose_account_does_not_exist(pool: PgPool) {
    let app = TestApp::new(pool);
    let token = app.token_for(Uuid::new_v4(), Role::User);

    let response = app.get("/auth/session", Some(&token)).await;

    response.assert_status(StatusCode::UNAUTHORIZED);
    assert!(response.detail().contains("n'existe pas"));
}

#[sqlx::test]
async fn session_refuses_a_refresh_token(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.seed("Sneaxiii", Role::User).await;
    let refresh = app.jwt.refresh_token(id).unwrap();

    app.get("/auth/session", Some(&refresh))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn refresh_returns_a_new_pair(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.seed("Sneaxiii", Role::User).await;
    let refresh = app.jwt.refresh_token(id).unwrap();

    let response = app
        .post("/auth/refresh", None, json!({ "refresh_token": refresh }))
        .await;

    response.assert_status(StatusCode::OK);
    let access = response.body["access_token"].as_str().unwrap().to_owned();
    app.get("/auth/session", Some(&access))
        .await
        .assert_status(StatusCode::OK);
}

#[sqlx::test]
async fn refresh_refuses_an_access_token(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, access) = app.seed_with_token("Sneaxiii", Role::User).await;

    app.post("/auth/refresh", None, json!({ "refresh_token": access }))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn refresh_refuses_an_unknown_account(pool: PgPool) {
    let app = TestApp::new(pool);
    let refresh = app.jwt.refresh_token(Uuid::new_v4()).unwrap();

    let response = app
        .post("/auth/refresh", None, json!({ "refresh_token": refresh }))
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);
    assert!(response.detail().contains("n'existe pas"));
}

#[sqlx::test]
async fn health_needs_no_token(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app.get("/health", None).await;

    response.assert_status(StatusCode::OK);
    assert_eq!(response.body["status"], "ok");
}

#[sqlx::test]
async fn dev_login_without_a_login_invents_one(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app.post("/auth/dev-login", None, json!({})).await;

    response.assert_status(StatusCode::OK);
    let token = response.body["access_token"].as_str().unwrap();
    let profile = app.get("/auth/session", Some(token)).await;

    let login = profile.body["login"].as_str().unwrap();
    assert!(
        login.chars().all(|c| c.is_ascii_alphanumeric()) && login.len() >= 3,
        "generated login is not usable: {login}"
    );
    assert_eq!(profile.body["role"], "user", "default role must stay user");
}

#[sqlx::test]
async fn a_generated_login_never_collides_with_an_existing_one(pool: PgPool) {
    let app = TestApp::new(pool);

    for _ in 0..15 {
        app.post("/auth/dev-login", None, json!({}))
            .await
            .assert_status(StatusCode::OK);
    }

    let distinct: i64 = sqlx::query_scalar(r#"SELECT count(DISTINCT login) FROM "user""#)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(distinct, 15, "two accounts share a login");
}

#[sqlx::test]
async fn a_role_can_be_requested_without_naming_the_account(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app
        .post("/auth/dev-login", None, json!({ "role": "super_admin" }))
        .await;

    response.assert_status(StatusCode::OK);
    let token = response.body["access_token"].as_str().unwrap();
    let profile = app.get("/auth/session", Some(token)).await;
    assert_eq!(profile.body["role"], "super_admin");
}

#[sqlx::test]
async fn a_supplied_login_is_still_validated(pool: PgPool) {
    let app = TestApp::new(pool);

    app.post("/auth/dev-login", None, json!({ "login": "no" }))
        .await
        .assert_status(StatusCode::UNPROCESSABLE_ENTITY);

    app.post("/auth/dev-login", None, json!({ "login": "drop-table" }))
        .await
        .assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}
