#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `/games/{id}/ws` as a socket: what it answers on joining, and what it pushes to the others.

mod common;

use std::time::Duration;

use common::{TestApp, answered};
use http::StatusCode;
use scout_beam::Config;
use scout_beam::engine::Layout;
use scout_beam::shared::role::Role;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test]
async fn joining_answers_the_game_as_it_reads_over_http(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;
    app.play(game_id, Some(&token), json!({ "type": "fire", "entry": 7 }))
        .await
        .assert_status(StatusCode::CREATED);

    let socket = app.join(game_id, Some(&token)).await.unwrap();

    let read = app.get(&format!("/games/{game_id}"), Some(&token)).await;
    assert_eq!(socket.joined, read.body);
    assert_eq!(socket.joined["shots"][0]["entry"], 7);
}

#[sqlx::test]
async fn a_new_shot_is_pushed_to_the_other_sockets_and_not_echoed(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;
    let mut shooter = app.join(game_id, Some(&token)).await.unwrap();
    let mut watcher = app.join(game_id, Some(&token)).await.unwrap();

    let fired = shooter
        .request(json!({ "type": "fire", "entry": 12 }))
        .await;

    fired.assert_status(StatusCode::CREATED);
    let pushed = answered(watcher.next().await.unwrap());
    assert_eq!(pushed.body, fired.body);
    shooter.assert_quiet().await;
}

#[sqlx::test]
async fn a_replayed_shot_is_pushed_to_nobody(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;
    let mut shooter = app.join(game_id, Some(&token)).await.unwrap();
    shooter
        .request(json!({ "type": "fire", "entry": 3 }))
        .await
        .assert_status(StatusCode::CREATED);
    let mut watcher = app.join(game_id, Some(&token)).await.unwrap();

    shooter
        .request(json!({ "type": "fire", "entry": 3 }))
        .await
        .assert_status(StatusCode::OK);

    watcher.assert_quiet().await;
}

#[sqlx::test]
async fn the_end_of_the_game_is_pushed_with_its_layout(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let solved_id = app.start_game(&token).await;
    let surrendered_id = app.start_game(&token).await;
    let mut solved_watcher = app.join(solved_id, Some(&token)).await.unwrap();
    let mut surrendered_watcher = app.join(surrendered_id, Some(&token)).await.unwrap();

    app.solve(solved_id, &token).await;
    app.play(surrendered_id, Some(&token), json!({ "type": "surrender" }))
        .await
        .assert_status(StatusCode::OK);

    for (watcher, status) in [
        (&mut solved_watcher, "solved"),
        (&mut surrendered_watcher, "surrendered"),
    ] {
        let pushed = answered(watcher.next().await.unwrap());
        pushed.assert_status(StatusCode::OK);
        assert_eq!(pushed.body["status"], status);
        assert!(pushed.body["layout"].is_object(), "{}", pushed.body);
    }
}

#[sqlx::test]
async fn a_wrong_proposition_is_pushed_to_nobody(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;
    let mut watcher = app.join(game_id, Some(&token)).await.unwrap();
    app.hide_layout(game_id, &layout_with_square("red")).await;
    let wrong = layout_with_square("blue");

    let proposed = app.propose(game_id, Some(&token), &wrong).await;

    assert_eq!(proposed.body["correct"], false, "{}", proposed.body);
    watcher.assert_quiet().await;
}

#[sqlx::test]
async fn an_unreadable_message_is_refused_and_the_socket_stays_open(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;
    let mut socket = app.join(game_id, Some(&token)).await.unwrap();

    for message in [json!({ "type": "jump" }), json!("fire")] {
        let refused = socket.request(message).await;
        refused.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(refused.body["action"], "message");
    }
    let again = socket
        .request(json!({ "type": "auth", "token": token }))
        .await;
    again.assert_status(StatusCode::BAD_REQUEST);

    socket
        .request(json!({ "type": "fire", "entry": 7 }))
        .await
        .assert_status(StatusCode::CREATED);
}

#[sqlx::test]
async fn a_refused_connection_says_why_and_closes(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, owner_token) = app.seed_with_token("owner", Role::User).await;
    let (_, intruder_token) = app.seed_with_token("intruder", Role::User).await;
    let game_id = app.start_game(&owner_token).await;

    for (token, status, code) in [
        (None, StatusCode::UNAUTHORIZED, 4401),
        (Some("not-a-token"), StatusCode::UNAUTHORIZED, 4401),
        (Some(intruder_token.as_str()), StatusCode::NOT_FOUND, 4404),
    ] {
        let Err(refused) = app.join(game_id, token).await else {
            panic!("{token:?} joined");
        };
        refused.assert_status(status);
        assert_eq!(refused.body["action"], "connect");
        let mut socket = app.join_raw(game_id, token).await;
        assert_eq!(socket.next().await.unwrap()["status"], status.as_u16());
        assert_eq!(socket.next().await, Err(Some(code)));
    }
}

#[sqlx::test]
async fn the_socket_closes_when_its_token_expires(pool: PgPool) {
    let config = Config {
        access_token_ttl: Duration::from_secs(1),
        ..Config::for_tests()
    };
    let app = TestApp::with_config(pool, config);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;
    let mut socket = app.join(game_id, Some(&token)).await.unwrap();

    let expired = answered(socket.next().await.unwrap());

    expired.assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(expired.body["action"], "connect");
    assert_eq!(socket.next().await, Err(Some(4401)));
}

/// Recolouring the square changes the beams it sends back: a wrong proposition, for sure.
fn layout_with_square(colour: &str) -> Layout {
    serde_json::from_value(json!({ "pieces": [
        { "shape": "square", "colour": colour, "x": 5, "y": 0, "rotation": 0 },
        { "shape": "bar", "colour": "red", "x": 7, "y": 3, "rotation": 1 },
        { "shape": "spike", "colour": "white", "x": 5, "y": 4, "rotation": 0 },
        { "shape": "corner", "colour": "blue", "x": 3, "y": 2, "rotation": 0 },
        { "shape": "diamond", "colour": "yellow", "x": 1, "y": 5, "rotation": 0 },
    ]}))
    .unwrap()
}
