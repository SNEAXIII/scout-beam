#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `/rooms/*` end to end: meeting through a code, a Race on one layout, and the end of the room.

mod common;

use common::{Response, TestApp};
use http::StatusCode;
use scout_beam::shared::role::Role;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

impl TestApp {
    async fn open_room(&self, token: &str, mode: &str) -> String {
        let created = self
            .post("/rooms", Some(token), json!({ "mode": mode }))
            .await;
        created.assert_status(StatusCode::CREATED);
        created.body["code"].as_str().unwrap().to_owned()
    }

    async fn join_room(&self, code: &str, token: &str) -> Response {
        self.post(&format!("/rooms/{code}/join"), Some(token), json!({}))
            .await
    }

    async fn room(&self, code: &str, token: &str) -> Response {
        let room = self.get(&format!("/rooms/{code}"), Some(token)).await;
        room.assert_status(StatusCode::OK);
        room
    }

    async fn game(&self, game_id: Uuid, token: &str) -> Response {
        let game = self.get(&format!("/games/{game_id}"), Some(token)).await;
        game.assert_status(StatusCode::OK);
        game
    }

    /// A Race hosted by alice and joined by bob: the room code and each one's game and token.
    async fn race(&self) -> (String, (Uuid, String), (Uuid, String)) {
        let (_, alice) = self.seed_with_token("alice", Role::User).await;
        let (_, bob) = self.seed_with_token("bob", Role::User).await;
        let code = self.open_room(&alice, "race").await;
        self.join_room(&code, &bob)
            .await
            .assert_status(StatusCode::OK);
        let alice_game = game_id(&self.room(&code, &alice).await);
        let bob_game = game_id(&self.room(&code, &bob).await);
        (code, (alice_game, alice), (bob_game, bob))
    }
}

fn game_id(room: &Response) -> Uuid {
    room.body["game_id"].as_str().unwrap().parse().unwrap()
}

#[sqlx::test]
async fn a_room_code_matches_the_alphabet(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("host", Role::User).await;

    for _ in 0..20 {
        let code = app.open_room(&token, "race").await;
        assert_eq!(code.len(), 6, "{code}");
        assert!(
            code.chars()
                .all(|c| "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".contains(c)),
            "{code}"
        );
    }
    let room = app.room(&app.open_room(&token, "duel").await, &token).await;
    assert_eq!(room.body["mode"], "duel");
    assert_eq!(room.body["status"], "waiting");
    assert_eq!(room.body["host"], "host");
}

#[sqlx::test]
async fn a_guest_gets_403_on_every_room_route(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, host) = app.seed_with_token("host", Role::User).await;
    let code = app.open_room(&host, "race").await;
    let (_, guest) = app.guest().await;

    let created = app
        .post("/rooms", Some(&guest), json!({ "mode": "race" }))
        .await;
    let read = app.get(&format!("/rooms/{code}"), Some(&guest)).await;
    let joined = app.join_room(&code, &guest).await;

    for refused in [created, read, joined] {
        refused.assert_status(StatusCode::FORBIDDEN);
    }
}

#[sqlx::test]
async fn joining_twice_or_as_the_host_is_a_conflict(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, host) = app.seed_with_token("host", Role::User).await;
    let (_, opponent) = app.seed_with_token("opponent", Role::User).await;
    let (_, third) = app.seed_with_token("third", Role::User).await;
    let code = app.open_room(&host, "duel").await;

    app.join_room(&code, &host)
        .await
        .assert_status(StatusCode::CONFLICT);
    let joined = app.join_room(&code, &opponent).await;
    joined.assert_status(StatusCode::OK);
    app.join_room(&code, &opponent)
        .await
        .assert_status(StatusCode::CONFLICT);
    app.join_room(&code, &third)
        .await
        .assert_status(StatusCode::CONFLICT);
    app.join_room("ZZZZZZ", &third)
        .await
        .assert_status(StatusCode::NOT_FOUND);

    // A Duel is composed before any game exists.
    assert_eq!(joined.body["status"], "composing");
    assert_eq!(joined.body["opponent"], "opponent");
    assert!(joined.body.get("game_id").is_none(), "{}", joined.body);
}

#[sqlx::test]
async fn a_race_gives_both_players_the_same_layout(pool: PgPool) {
    let app = TestApp::new(pool);
    let (code, (alice_game, alice), (bob_game, bob)) = app.race().await;
    let alice_view = app.game(alice_game, &alice).await;
    assert_eq!(alice_view.body["status"], "in_progress");
    assert_eq!(alice_view.body["room_code"], code);
    assert_eq!(app.room(&code, &bob).await.body["status"], "playing");

    app.solve(alice_game, &alice).await;

    let alice_view = app.game(alice_game, &alice).await;
    let bob_view = app.game(bob_game, &bob).await;
    assert_ne!(alice_game, bob_game);
    assert!(alice_view.body["layout"].is_object());
    assert_eq!(alice_view.body["layout"], bob_view.body["layout"]);
}

#[sqlx::test]
async fn the_first_correct_proposition_wins_the_room(pool: PgPool) {
    let app = TestApp::new(pool);
    let (code, (alice_game, alice), (bob_game, bob)) = app.race().await;

    app.solve(alice_game, &alice).await;

    let room = app.room(&code, &bob).await;
    assert_eq!(room.body["status"], "finished");
    assert_eq!(room.body["winner"], "alice");
    assert_eq!(app.game(alice_game, &alice).await.body["status"], "solved");
    assert_eq!(app.game(bob_game, &bob).await.body["status"], "lost");
    app.play(bob_game, Some(&bob), json!({ "type": "fire", "entry": 7 }))
        .await
        .assert_status(StatusCode::CONFLICT);
    let games = app.get("/games", Some(&bob)).await;
    assert_eq!(games.body[0]["id"], bob_game.to_string());
    assert_eq!(games.body[0]["status"], "lost");
    assert_eq!(games.body[0]["mode"], "race");
    assert_eq!(games.body[0]["opponent"], "alice");
}

#[sqlx::test]
async fn a_surrender_makes_the_opponent_win(pool: PgPool) {
    let app = TestApp::new(pool);
    let (code, (alice_game, alice), (bob_game, bob)) = app.race().await;

    app.play(bob_game, Some(&bob), json!({ "type": "surrender" }))
        .await
        .assert_status(StatusCode::OK);

    let room = app.room(&code, &alice).await;
    assert_eq!(room.body["status"], "finished");
    assert_eq!(room.body["winner"], "alice");
    let alice_view = app.game(alice_game, &alice).await;
    assert_eq!(alice_view.body["status"], "solved");
    assert!(alice_view.body["layout"].is_object());
    assert_eq!(app.game(bob_game, &bob).await.body["status"], "surrendered");
}

#[sqlx::test]
async fn a_non_member_sees_only_the_public_fields(pool: PgPool) {
    let app = TestApp::new(pool);
    let (code, (alice_game, alice), _) = app.race().await;
    app.solve(alice_game, &alice).await;
    let (_, stranger) = app.seed_with_token("stranger", Role::User).await;

    let seen = app.room(&code, &stranger).await;

    assert_eq!(
        seen.body,
        json!({ "code": code, "mode": "race", "status": "finished", "host": "alice" })
    );
    app.get("/rooms/ZZZZZZ", Some(&stranger))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}
