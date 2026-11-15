#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `/auth/*` — minting a session, reading it, renewing it.

mod common;

use common::{TestApp, login_body};
use http::StatusCode;
use scout_beam::Config;
use scout_beam::config::Environment;
use scout_beam::repositories::user_repository;
use scout_beam::shared::role::Role;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// The state a lost race leaves behind: seeding the row between lookup and insert *is* that state.
#[sqlx::test]
async fn a_login_taken_between_the_lookup_and_the_insert_is_not_an_error(pool: PgPool) {
    let app = TestApp::new(pool);
    app.seed("Sneaxiii", Role::User).await;

    let outcome = user_repository::insert_if_absent(&app.pool, "Sneaxiii", Role::User).await;

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
    assert_eq!(response.body["login"], "Sneaxiii");
    assert!(response.cookie("scout_beam_access").is_some());
    assert!(response.cookie("scout_beam_refresh").is_some());
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
    assert_eq!(response.code(), "account_not_found", "{}", response.body);
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
async fn refresh_works_from_the_refresh_cookie_alone(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.seed("Sneaxiii", Role::User).await;
    let refresh = app.jwt.refresh_token(id).unwrap();

    let response = app
        .with_cookie(
            http::Method::POST,
            "/auth/refresh",
            &format!("scout_beam_refresh={refresh}"),
        )
        .await;

    response.assert_status(StatusCode::OK);
    assert_eq!(response.body["login"], "Sneaxiii");
    assert!(response.body.get("access_token").is_none());
    assert!(response.cookie("scout_beam_refresh").is_some());
    let access = response.access_token();
    app.get("/auth/session", Some(&access))
        .await
        .assert_status(StatusCode::OK);
}

#[sqlx::test]
async fn refresh_without_its_cookie_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, access) = app.seed_with_token("Sneaxiii", Role::User).await;

    app.post("/auth/refresh", Some(&access), json!({}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn refresh_refuses_an_access_token(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, access) = app.seed_with_token("Sneaxiii", Role::User).await;

    app.with_cookie(
        http::Method::POST,
        "/auth/refresh",
        &format!("scout_beam_refresh={access}"),
    )
    .await
    .assert_status(StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn refresh_refuses_an_unknown_account(pool: PgPool) {
    let app = TestApp::new(pool);
    let refresh = app.jwt.refresh_token(Uuid::new_v4()).unwrap();

    let response = app
        .with_cookie(
            http::Method::POST,
            "/auth/refresh",
            &format!("scout_beam_refresh={refresh}"),
        )
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(response.code(), "account_not_found", "{}", response.body);
}

#[sqlx::test]
async fn a_sign_in_answers_the_profile_and_sets_both_cookies_out_of_scripts_reach(pool: PgPool) {
    let app = TestApp::new(pool);

    let signed = app
        .post(
            "/auth/register",
            None,
            credentials("Sneaxiii", "correct horse"),
        )
        .await;

    signed.assert_status(StatusCode::CREATED);
    let body = signed.body.to_string();
    assert!(
        !body.contains("token"),
        "a token leaked into the body: {body}"
    );
    assert_eq!(signed.body["login"], "Sneaxiii");
    for name in ["scout_beam_access", "scout_beam_refresh"] {
        let cookie = signed.set_cookie(name).unwrap();
        for part in ["HttpOnly", "SameSite=Strict", "Path=/", "Max-Age="] {
            assert!(cookie.contains(part), "{part} missing from {cookie}");
        }
        assert!(
            !cookie.contains("Secure"),
            "dev serves plain HTTP: {cookie}"
        );
    }
}

#[sqlx::test]
async fn cookies_are_secure_outside_dev(pool: PgPool) {
    let config = Config {
        environment: Environment::Prod,
        ..Config::for_tests()
    };
    let app = TestApp::with_config(pool, config);

    let signed = app
        .post(
            "/auth/register",
            None,
            credentials("Sneaxiii", "correct horse"),
        )
        .await;

    assert!(
        signed
            .set_cookie("scout_beam_access")
            .unwrap()
            .contains("Secure")
    );
}

#[sqlx::test]
async fn a_cookie_only_request_reaches_the_session(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, access) = app.seed_with_token("Sneaxiii", Role::User).await;

    let session = app
        .with_cookie(
            http::Method::GET,
            "/auth/session",
            &format!("theme=dark; scout_beam_access={access}"),
        )
        .await;

    session.assert_status(StatusCode::OK);
    assert_eq!(session.body["login"], "Sneaxiii");
}

#[sqlx::test]
async fn logout_clears_both_cookies(pool: PgPool) {
    let app = TestApp::new(pool);

    let out = app.post("/auth/logout", None, json!({})).await;

    out.assert_status(StatusCode::NO_CONTENT);
    for name in ["scout_beam_access", "scout_beam_refresh"] {
        let cookie = out.set_cookie(name).unwrap();
        assert!(cookie.starts_with(&format!("{name}=;")), "{cookie}");
        assert!(cookie.contains("Max-Age=0"), "{cookie}");
    }
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
    let token = &response.access_token();
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
    let token = &response.access_token();
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

fn credentials(login: &str, password: &str) -> serde_json::Value {
    json!({ "login": login, "password": password })
}

#[sqlx::test]
async fn register_then_login_returns_a_working_token_pair(pool: PgPool) {
    let app = TestApp::new(pool);

    let registered = app
        .post(
            "/auth/register",
            None,
            credentials("Sneaxiii", "correct horse"),
        )
        .await;
    registered.assert_status(StatusCode::CREATED);

    let logged = app
        .post(
            "/auth/login",
            None,
            credentials("Sneaxiii", "correct horse"),
        )
        .await;
    logged.assert_status(StatusCode::OK);
    let access = &logged.access_token();
    app.get("/auth/session", Some(access))
        .await
        .assert_status(StatusCode::OK);
}

#[sqlx::test]
async fn a_wrong_password_and_an_unknown_login_get_the_same_answer(pool: PgPool) {
    let app = TestApp::new(pool);
    app.post(
        "/auth/register",
        None,
        credentials("Sneaxiii", "correct horse"),
    )
    .await;

    let wrong = app
        .post(
            "/auth/login",
            None,
            credentials("Sneaxiii", "battery staple"),
        )
        .await;
    let unknown = app
        .post("/auth/login", None, credentials("Nobody", "battery staple"))
        .await;

    wrong.assert_status(StatusCode::UNAUTHORIZED);
    unknown.assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(wrong.body, unknown.body);
}

#[sqlx::test]
async fn an_account_without_a_password_cannot_be_signed_into_by_password(pool: PgPool) {
    let app = TestApp::new(pool);
    app.seed("Sneaxiii", Role::User).await;

    app.post(
        "/auth/login",
        None,
        credentials("Sneaxiii", "correct horse"),
    )
    .await
    .assert_status(StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn a_taken_login_answers_409(pool: PgPool) {
    let app = TestApp::new(pool);
    app.post(
        "/auth/register",
        None,
        credentials("Sneaxiii", "correct horse"),
    )
    .await
    .assert_status(StatusCode::CREATED);

    app.post(
        "/auth/register",
        None,
        credentials("Sneaxiii", "another one"),
    )
    .await
    .assert_status(StatusCode::CONFLICT);
}

#[sqlx::test]
async fn a_seven_character_password_answers_400(pool: PgPool) {
    let app = TestApp::new(pool);

    app.post("/auth/register", None, credentials("Sneaxiii", "1234567"))
        .await
        .assert_status(StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn the_password_hash_is_stored_and_never_the_password(pool: PgPool) {
    let app = TestApp::new(pool);

    app.post(
        "/auth/register",
        None,
        credentials("Sneaxiii", "correct horse"),
    )
    .await
    .assert_status(StatusCode::CREATED);

    let stored: Option<String> =
        sqlx::query_scalar(r#"SELECT password_hash FROM "user" WHERE login = $1"#)
            .bind("Sneaxiii")
            .fetch_one(&app.pool)
            .await
            .unwrap();
    let stored = stored.unwrap();
    assert!(
        stored.starts_with("$argon2id$"),
        "not an Argon2id hash: {stored}"
    );
    assert!(!stored.contains("correct horse"));
}

#[sqlx::test]
async fn a_sixth_sign_in_attempt_within_a_minute_answers_429(pool: PgPool) {
    let app = TestApp::new(pool);
    app.post(
        "/auth/register",
        None,
        credentials("Sneaxiii", "correct horse"),
    )
    .await;

    for _ in 0..4 {
        app.post(
            "/auth/login",
            None,
            credentials("Sneaxiii", "battery staple"),
        )
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    }
    app.post(
        "/auth/login",
        None,
        credentials("Sneaxiii", "battery staple"),
    )
    .await
    .assert_status(StatusCode::TOO_MANY_REQUESTS);
}

#[sqlx::test]
async fn a_guest_gets_a_token_and_is_flagged_in_its_session(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.guest().await;

    let session = app.get("/auth/session", Some(&token)).await;

    session.assert_status(StatusCode::OK);
    assert_eq!(session.body["guest"], true);
    let login = session.body["login"].as_str().unwrap();
    assert!(login.chars().all(|c| c.is_ascii_alphanumeric()), "{login}");
}

#[sqlx::test]
async fn registering_with_a_guest_bearer_converts_that_guest_and_keeps_its_games(pool: PgPool) {
    let app = TestApp::new(pool);
    let (guest_id, guest_token) = app.guest().await;
    let game_id: Uuid = sqlx::query_scalar(
        "INSERT INTO game (user_id, layout) VALUES ($1, '[]'::jsonb) RETURNING id",
    )
    .bind(guest_id)
    .fetch_one(&app.pool)
    .await
    .unwrap();

    let registered = app
        .post(
            "/auth/register",
            Some(&guest_token),
            credentials("Sneaxiii", "correct horse"),
        )
        .await;
    registered.assert_status(StatusCode::CREATED);

    let (id, login, guest): (Uuid, String, bool) =
        sqlx::query_as(r#"SELECT id, login, guest FROM "user""#)
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!((id, login.as_str(), guest), (guest_id, "Sneaxiii", false));

    let owner: Uuid = sqlx::query_scalar("SELECT user_id FROM game WHERE id = $1")
        .bind(game_id)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(owner, guest_id);

    let access = &registered.access_token();
    let session = app.get("/auth/session", Some(access)).await;
    assert_eq!(session.body["guest"], false);
    app.post(
        "/auth/login",
        None,
        credentials("Sneaxiii", "correct horse"),
    )
    .await
    .assert_status(StatusCode::OK);
}

#[sqlx::test]
async fn registering_with_a_non_guest_bearer_creates_a_separate_account(pool: PgPool) {
    let app = TestApp::new(pool);
    let (member_id, member_token) = app.seed_with_token("Member", Role::User).await;

    app.post(
        "/auth/register",
        Some(&member_token),
        credentials("Sneaxiii", "correct horse"),
    )
    .await
    .assert_status(StatusCode::CREATED);

    let login: String = sqlx::query_scalar(r#"SELECT login FROM "user" WHERE id = $1"#)
        .bind(member_id)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(login, "Member", "the bearer's account must stay as it is");
}

#[sqlx::test]
async fn registering_with_an_invalid_bearer_ignores_it(pool: PgPool) {
    let app = TestApp::new(pool);

    app.post(
        "/auth/register",
        Some("not-a-jwt"),
        credentials("Sneaxiii", "correct horse"),
    )
    .await
    .assert_status(StatusCode::CREATED);
}

#[sqlx::test]
async fn a_guest_bearer_on_login_leaves_the_guest_untouched(pool: PgPool) {
    let app = TestApp::new(pool);
    app.post(
        "/auth/register",
        None,
        credentials("Sneaxiii", "correct horse"),
    )
    .await;
    let (guest_id, guest_token) = app.guest().await;

    app.post(
        "/auth/login",
        Some(&guest_token),
        credentials("Sneaxiii", "correct horse"),
    )
    .await
    .assert_status(StatusCode::OK);

    let still_guest: bool = sqlx::query_scalar(r#"SELECT guest FROM "user" WHERE id = $1"#)
        .bind(guest_id)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert!(still_guest);
}

#[sqlx::test]
async fn a_guest_cannot_hold_a_password(pool: PgPool) {
    let app = TestApp::new(pool);
    let (guest_id, _) = app.guest().await;

    let refused = sqlx::query(r#"UPDATE "user" SET password_hash = 'x' WHERE id = $1"#)
        .bind(guest_id)
        .execute(&app.pool)
        .await;

    assert!(refused.is_err(), "the CHECK constraint must refuse it");
}
