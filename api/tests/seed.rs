#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `service::seed` — the door the `seed-user` binary goes through.
//!
//! Called directly, with no router and no application state, as the CLI does.

mod common;

use common::TestApp;
use scout_beam::domains::user::UserError;
use scout_beam::domains::user::service::{self, SeedOutcome};
use scout_beam::shared::role::Role;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test]
async fn an_unknown_login_is_created_with_the_wanted_role(pool: PgPool) {
    let (user, outcome) = service::seed(&pool, "Sneaxiii", Role::SuperAdmin)
        .await
        .unwrap();

    assert_eq!(outcome, SeedOutcome::Created);
    assert_eq!(user.login, "Sneaxiii");
    assert_eq!(user.role, Role::SuperAdmin);
}

#[sqlx::test]
async fn running_it_twice_changes_nothing(pool: PgPool) {
    service::seed(&pool, "Sneaxiii", Role::Admin).await.unwrap();

    let (_, outcome) = service::seed(&pool, "Sneaxiii", Role::Admin).await.unwrap();

    assert_eq!(outcome, SeedOutcome::Unchanged);
    let count: i64 = sqlx::query_scalar(r#"SELECT count(*) FROM "user""#)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1, "a second run must not duplicate the account");
}

#[sqlx::test]
async fn an_existing_login_is_promoted_rather_than_duplicated(pool: PgPool) {
    service::seed(&pool, "Sneaxiii", Role::User).await.unwrap();

    let (user, outcome) = service::seed(&pool, "Sneaxiii", Role::SuperAdmin)
        .await
        .unwrap();

    assert_eq!(outcome, SeedOutcome::RoleChanged { from: Role::User });
    assert_eq!(user.role, Role::SuperAdmin);
}

#[sqlx::test]
async fn a_promoted_account_signs_in_with_its_new_role(pool: PgPool) {
    let app = TestApp::new(pool);
    service::seed(&app.pool, "Sneaxiii", Role::User)
        .await
        .unwrap();
    service::seed(&app.pool, "Sneaxiii", Role::Admin)
        .await
        .unwrap();

    let response = app
        .post("/auth/dev-login", None, json!({ "login": "Sneaxiii" }))
        .await;

    response.assert_status(http::StatusCode::OK);
    let token = response.body["access_token"].as_str().unwrap();
    let profile = app.get("/auth/session", Some(token)).await;
    assert_eq!(profile.body["role"], "admin");
}

#[sqlx::test]
async fn a_login_the_api_would_refuse_is_refused_here_too(pool: PgPool) {
    for candidate in ["ab", "with space", "drop-table", "accentué"] {
        let error = service::seed(&pool, candidate, Role::User)
            .await
            .unwrap_err();

        assert!(
            matches!(error, UserError::InvalidLogin),
            "{candidate} was not rejected: {error}"
        );
    }

    let count: i64 = sqlx::query_scalar(r#"SELECT count(*) FROM "user""#)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0, "an invalid login must not reach the table");
}
