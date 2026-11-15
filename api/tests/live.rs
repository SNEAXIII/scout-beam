#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `/games/{id}/ws` as a socket: what it answers on joining, and what it pushes to the others.

mod common;

use std::time::Duration;

use common::{GameSocket, TestApp, answered};
use http::StatusCode;
use http::header::{COOKIE, ORIGIN};
use scout_beam::Config;
use scout_beam::engine::{Layout, draw_layout};
use scout_beam::shared::role::Role;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// A room player with a socket open on their game.
struct Player {
    game: Uuid,
    token: String,
    socket: GameSocket,
}

impl TestApp {
    /// A playing room of that mode, hosted by alice and joined by bob; a Duel composed at random.
    async fn playing_room(&self, mode: &str) -> (Player, Player) {
        let (_, alice) = self.seed_with_token("alice", Role::User).await;
        let (_, bob) = self.seed_with_token("bob", Role::User).await;
        let created = self
            .post("/rooms", Some(&alice), json!({ "mode": mode }))
            .await;
        let code = created.body["code"].as_str().unwrap().to_owned();
        self.post(&format!("/rooms/{code}/join"), Some(&bob), json!({}))
            .await
            .assert_status(StatusCode::OK);
        if mode == "duel" {
            for token in [&alice, &bob] {
                let layout = serde_json::to_value(draw_layout(&mut rand::rng()).unwrap()).unwrap();
                self.post(&format!("/rooms/{code}/layout"), Some(token), layout)
                    .await
                    .assert_status(StatusCode::CREATED);
            }
        }
        (
            self.player(&code, alice).await,
            self.player(&code, bob).await,
        )
    }

    async fn player(&self, code: &str, token: String) -> Player {
        let room = self.get(&format!("/rooms/{code}"), Some(&token)).await;
        let game = room.body["game_id"].as_str().unwrap().parse().unwrap();
        let socket = self.join(game, Some(&token)).await.unwrap();
        Player {
            game,
            token,
            socket,
        }
    }
}

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

#[sqlx::test]
async fn a_socket_opened_from_a_foreign_origin_is_refused_before_the_upgrade(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;

    let refused = app
        .join_with(
            game_id,
            [
                (ORIGIN, "https://evil.example".to_owned()),
                (COOKIE, format!("scout_beam_access={token}")),
            ],
        )
        .await;

    assert_eq!(refused.err(), Some(StatusCode::FORBIDDEN));
}

#[sqlx::test]
async fn the_fronts_origin_and_the_access_cookie_alone_open_the_socket(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;

    let mut socket = app
        .join_with(
            game_id,
            [
                (ORIGIN, "http://localhost:3001".to_owned()),
                (COOKIE, format!("scout_beam_access={token}")),
            ],
        )
        .await
        .unwrap();

    let joined = answered(socket.next().await.unwrap());
    joined.assert_status(StatusCode::OK);
    assert_eq!(joined.body["id"], game_id.to_string());
}

#[sqlx::test]
async fn the_opponent_is_pushed_a_shot_count_and_never_the_shot(pool: PgPool) {
    let app = TestApp::new(pool);
    let (mut alice, mut bob) = app.playing_room("race").await;
    let mut alice_tab = app.join(alice.game, Some(&alice.token)).await.unwrap();

    for entry in [7, 12, 7] {
        alice
            .socket
            .request(json!({ "type": "fire", "entry": entry }))
            .await;
    }

    for shots in [1, 2] {
        let pushed = bob.socket.next().await.unwrap();
        assert_eq!(pushed, json!({ "type": "opponent", "shots": shots }));
    }
    bob.socket.assert_quiet().await;
    for _ in 0..2 {
        assert_eq!(alice_tab.next().await.unwrap()["type"], "shot");
    }
    alice_tab.assert_quiet().await;
    alice.socket.assert_quiet().await;
}

#[sqlx::test]
async fn both_players_sockets_are_pushed_their_own_game_when_the_room_ends(pool: PgPool) {
    let app = TestApp::new(pool);
    let (mut alice, mut bob) = app.playing_room("race").await;

    app.solve(alice.game, &alice.token).await;

    for (player, status) in [(&mut alice, "solved"), (&mut bob, "lost")] {
        let pushed = answered(player.socket.next().await.unwrap());
        pushed.assert_status(StatusCode::OK);
        assert_eq!(pushed.body["id"], player.game.to_string());
        assert_eq!(pushed.body["status"], status);
        assert!(pushed.body["layout"].is_object(), "{}", pushed.body);
        player.socket.assert_quiet().await;
    }
}

#[sqlx::test]
async fn a_surrender_pushes_the_opponent_their_win(pool: PgPool) {
    let app = TestApp::new(pool);
    let (mut alice, mut bob) = app.playing_room("race").await;

    let surrendered = bob.socket.request(json!({ "type": "surrender" })).await;

    assert_eq!(surrendered.body["status"], "surrendered");
    let pushed = answered(alice.socket.next().await.unwrap());
    assert_eq!(pushed.body["status"], "solved");
    bob.socket.assert_quiet().await;
}

#[sqlx::test]
async fn a_room_games_messages_carry_its_pace_by_the_servers_clock(pool: PgPool) {
    let app = TestApp::new(pool);
    let (mut alice, _) = app.playing_room("duel").await;
    let mut alice_tab = app.join(alice.game, Some(&alice.token)).await.unwrap();
    assert!(alice.socket.joined["elapsed_ms"].is_u64());
    assert_eq!(alice.socket.joined["proposition_in_ms"], 0);

    alice
        .socket
        .send(json!({ "type": "fire", "entry": 7 }))
        .await;
    let shot = alice.socket.next().await.unwrap();
    assert_eq!(shot["type"], "shot");
    assert!(shot["elapsed_ms"].is_u64(), "{shot}");
    assert_eq!(alice_tab.next().await.unwrap(), shot);

    let mut proposition = serde_json::to_value(draw_layout(&mut rand::rng()).unwrap()).unwrap();
    proposition["type"] = json!("propose");
    alice.socket.send(proposition.clone()).await;
    let verdict = alice.socket.next().await.unwrap();
    assert_eq!(verdict["correct"], false, "{verdict}");
    assert_eq!(verdict["proposition_in_ms"], 5_000);

    let refused = alice.socket.request(proposition).await;
    refused.assert_status(StatusCode::CONFLICT);
    assert!(refused.body["retry_after_ms"].as_u64().unwrap() > 4_000);

    let (_, token) = app.seed_with_token("solo", Role::User).await;
    let solo = app.join(app.start_game(&token).await, Some(&token)).await;
    assert!(solo.unwrap().joined.get("elapsed_ms").is_none());
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
