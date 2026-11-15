#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `/users/me` — the caller changing their pseudo.

mod common;

use common::{TestApp, login_body};
use http::StatusCode;
use scout_beam::shared::role::Role;
use sqlx::PgPool;

#[sqlx::test]
async fn renaming_changes_the_login_and_keeps_the_session(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("Sneaxiii", Role::User).await;

    let response = app
        .patch("/users/me", Some(&token), Some(login_body("Scout42")))
        .await;

    response.assert_status(StatusCode::OK);
    assert_eq!(response.body["login"], "Scout42");
    let session = app.get("/auth/session", Some(&token)).await;
    session.assert_status(StatusCode::OK);
    assert_eq!(session.body["login"], "Scout42");
}

#[sqlx::test]
async fn renaming_to_ones_own_login_is_accepted(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("Sneaxiii", Role::User).await;

    app.patch("/users/me", Some(&token), Some(login_body("Sneaxiii")))
        .await
        .assert_status(StatusCode::OK);
}

#[sqlx::test]
async fn a_login_held_by_someone_else_is_a_conflict(pool: PgPool) {
    let app = TestApp::new(pool);
    app.seed("Taken", Role::User).await;
    let (_, token) = app.seed_with_token("Sneaxiii", Role::User).await;

    app.patch("/users/me", Some(&token), Some(login_body("Taken")))
        .await
        .assert_status(StatusCode::CONFLICT);
}

#[sqlx::test]
async fn an_invalid_login_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("Sneaxiii", Role::User).await;

    for login in ["ab", "with space", "Snéaxiii", &"a".repeat(31)] {
        app.patch("/users/me", Some(&token), Some(login_body(login)))
            .await
            .assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[sqlx::test]
async fn renaming_needs_a_session(pool: PgPool) {
    let app = TestApp::new(pool);

    app.patch("/users/me", None, Some(login_body("Scout42")))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}
