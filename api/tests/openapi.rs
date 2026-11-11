#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The published OpenAPI document.
//!
//! Hand-written in `#[utoipa::path]` attributes: these tests keep it from drifting from the router.

mod common;

use common::TestApp;
use http::StatusCode;
use sqlx::PgPool;

const EXPECTED_PATHS: [(&str, &str); 9] = [
    ("/health", "get"),
    ("/auth/dev-login", "post"),
    ("/auth/session", "get"),
    ("/auth/refresh", "post"),
    ("/games", "post"),
    ("/games", "get"),
    ("/games/{game_id}", "get"),
    ("/games/{game_id}/shots", "post"),
    ("/games/{game_id}/propositions", "post"),
];

#[sqlx::test]
async fn every_route_is_documented(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app.get("/api-docs/openapi.json", None).await;

    response.assert_status(StatusCode::OK);
    let paths = &response.body["paths"];
    for (path, method) in EXPECTED_PATHS {
        assert!(
            paths[path][method].is_object(),
            "{method} {path} is missing from the OpenAPI document"
        );
    }
}

#[sqlx::test]
async fn the_document_declares_nothing_extra(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app.get("/api-docs/openapi.json", None).await;

    let documented = response.body["paths"].as_object().unwrap().len();
    let expected_paths: std::collections::BTreeSet<&str> =
        EXPECTED_PATHS.iter().map(|(path, _)| *path).collect();
    assert_eq!(
        documented,
        expected_paths.len(),
        "a path was documented without being added to this test"
    );
}

#[sqlx::test]
async fn protected_routes_declare_the_bearer_scheme(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app.get("/api-docs/openapi.json", None).await;

    assert!(
        response.body["components"]["securitySchemes"]["bearer"].is_object(),
        "the bearer scheme is not declared"
    );
    // A route behind an extractor must say so, or "try it out" sends no token.
    assert!(
        response.body["paths"]["/auth/session"]["get"]["security"].is_array(),
        "/auth/session does not require a token in the document"
    );
    // …and an open one must not.
    assert!(
        response.body["paths"]["/auth/dev-login"]["post"]["security"].is_null(),
        "/auth/dev-login should not ask for a token"
    );
}

#[sqlx::test]
async fn the_document_is_titled_after_the_crate(pool: PgPool) {
    let app = TestApp::new(pool);

    let response = app.get("/api-docs/openapi.json", None).await;

    assert_eq!(response.body["info"]["title"], "scout-beam");
}

#[sqlx::test]
async fn the_swagger_page_is_served(pool: PgPool) {
    let app = TestApp::new(pool);

    // The UI itself redirects to its index; either answer means it is mounted.
    let response = app.get("/docs", None).await;

    assert!(
        response.status.is_success() || response.status.is_redirection(),
        "unexpected status for /docs: {}",
        response.status
    );
}
