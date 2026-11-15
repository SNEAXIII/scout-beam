#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! A Duel end to end: each player hides a layout for the other, then priority passes between them.

mod common;

use common::{Response, TestApp};
use http::StatusCode;
use scout_beam::engine::colour::Colour;
use scout_beam::engine::{Layout, draw_layout};
use scout_beam::shared::role::Role;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

/// One player of a Duel: their token and, once it started, their game.
struct Player {
    token: String,
    game: Uuid,
}

impl TestApp {
    /// A Duel hosted by alice, joined by bob, still composing.
    async fn composing_duel(&self) -> (String, String, String) {
        let (_, alice) = self.seed_with_token("alice", Role::User).await;
        let (_, bob) = self.seed_with_token("bob", Role::User).await;
        let created = self
            .post("/rooms", Some(&alice), json!({ "mode": "duel" }))
            .await;
        created.assert_status(StatusCode::CREATED);
        let code = created.body["code"].as_str().unwrap().to_owned();
        self.post(&format!("/rooms/{code}/join"), Some(&bob), json!({}))
            .await
            .assert_status(StatusCode::OK);
        (code, alice, bob)
    }

    async fn hide(&self, code: &str, token: &str, layout: &Layout) -> Response {
        self.post(
            &format!("/rooms/{code}/layout"),
            Some(token),
            serde_json::to_value(layout).unwrap(),
        )
        .await
    }

    /// A playing Duel where alice has already fired 17 entries, so her next action leaves bob
    /// 10 s × 18 / 36 = 5 s of priority.
    async fn playing_duel(&self) -> (String, Player, Player) {
        let (code, alice, bob) = self.composing_duel().await;
        self.hide(&code, &alice, &hidden()).await;
        let started = self.hide(&code, &bob, &hidden()).await;
        started.assert_status(StatusCode::CREATED);
        let alice_game = self.game_in(&code, &alice).await;
        let bob_game = started.body["game_id"].as_str().unwrap().parse().unwrap();
        sqlx::query(
            "INSERT INTO shot (game_id, entry, exit_entry, colour)
             SELECT $1, entry, entry, 'none' FROM generate_series(20, 36) entry",
        )
        .bind(alice_game)
        .execute(&self.pool)
        .await
        .unwrap();
        (
            code,
            Player {
                token: alice,
                game: alice_game,
            },
            Player {
                token: bob,
                game: bob_game,
            },
        )
    }

    async fn game_in(&self, code: &str, token: &str) -> Uuid {
        let room = self.get(&format!("/rooms/{code}"), Some(token)).await;
        room.body["game_id"].as_str().unwrap().parse().unwrap()
    }

    async fn fire(&self, player: &Player, entry: u8) -> Response {
        self.play(
            player.game,
            Some(&player.token),
            json!({ "type": "fire", "entry": entry }),
        )
        .await
    }
}

/// A drawn layout: valid, with red, blue and yellow.
fn hidden() -> Layout {
    draw_layout(&mut rand::rng()).unwrap()
}

fn retry_after_ms(body: &Value) -> u64 {
    body["retry_after_ms"].as_u64().unwrap()
}

#[sqlx::test]
async fn a_layout_without_yellow_is_refused_with_the_rule_broken(pool: PgPool) {
    let app = TestApp::new(pool);
    let (code, alice, _) = app.composing_duel().await;
    let mut without_yellow = hidden();
    for piece in &mut without_yellow.pieces {
        if piece.colour == Colour::Yellow {
            piece.colour = Colour::White;
        }
    }
    let without_diamond = Layout {
        pieces: hidden().pieces[..4].to_vec(),
    };

    let refused = app.hide(&code, &alice, &without_yellow).await;
    refused.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    assert!(refused.detail().contains("pièce jaune"), "{}", refused.body);
    app.hide(&code, &alice, &without_diamond)
        .await
        .assert_status(StatusCode::UNPROCESSABLE_ENTITY);

    // Nothing was stored: the valid layout still goes in.
    let hidden = app.hide(&code, &alice, &hidden()).await;
    hidden.assert_status(StatusCode::CREATED);
    assert_eq!(hidden.body["status"], "composing");
}

#[sqlx::test]
async fn a_layout_is_hidden_once_by_a_player_of_a_composing_duel(pool: PgPool) {
    let app = TestApp::new(pool);
    let (code, alice, _) = app.composing_duel().await;
    let (_, stranger) = app.seed_with_token("stranger", Role::User).await;
    let (_, guest) = app.guest().await;
    let race = app
        .post("/rooms", Some(&alice), json!({ "mode": "race" }))
        .await;
    let race = race.body["code"].as_str().unwrap();

    let first = app.hide(&code, &alice, &hidden()).await;
    first.assert_status(StatusCode::CREATED);
    assert!(!first.body.to_string().contains("pieces"), "{}", first.body);
    app.hide(&code, &alice, &hidden())
        .await
        .assert_status(StatusCode::CONFLICT);
    app.hide(&code, &stranger, &hidden())
        .await
        .assert_status(StatusCode::CONFLICT);
    app.hide(race, &alice, &hidden())
        .await
        .assert_status(StatusCode::CONFLICT);
    app.hide(&code, &guest, &hidden())
        .await
        .assert_status(StatusCode::FORBIDDEN);
    app.hide("ZZZZZZ", &alice, &hidden())
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn each_players_game_holds_the_others_layout(pool: PgPool) {
    let app = TestApp::new(pool);
    let (code, alice, bob) = app.composing_duel().await;
    let (alice_layout, bob_layout) = (hidden(), hidden());
    assert_ne!(alice_layout, bob_layout);

    app.hide(&code, &alice, &alice_layout).await;
    let started = app.hide(&code, &bob, &bob_layout).await;

    started.assert_status(StatusCode::CREATED);
    assert_eq!(started.body["status"], "playing");
    let alice_game = app.game_in(&code, &alice).await;
    let bob_game = app.game_in(&code, &bob).await;
    assert_eq!(app.stored_layout(alice_game).await, bob_layout);
    assert_eq!(app.stored_layout(bob_game).await, alice_layout);
    let game = app.get(&format!("/games/{alice_game}"), Some(&alice)).await;
    assert!(game.body.get("layout").is_none(), "{}", game.body);
    app.hide(&code, &alice, &hidden())
        .await
        .assert_status(StatusCode::CONFLICT);
}

#[sqlx::test]
async fn an_immediate_second_action_at_18_entries_is_refused_for_about_5_s(pool: PgPool) {
    let app = TestApp::new(pool);
    let (code, alice, bob) = app.playing_duel().await;

    app.fire(&alice, 1).await.assert_status(StatusCode::CREATED);

    let refused = app.fire(&alice, 2).await;
    refused.assert_status(StatusCode::CONFLICT);
    let left = retry_after_ms(&refused.body);
    assert!((4_000..=5_000).contains(&left), "{left}");
    let proposed = app.propose(alice.game, Some(&alice.token), &hidden()).await;
    proposed.assert_status(StatusCode::CONFLICT);
    assert!(retry_after_ms(&proposed.body) > 0);

    let seen = app.get(&format!("/rooms/{code}"), Some(&alice.token)).await;
    assert_eq!(seen.body["last_actor"], "alice");
    assert!((4_000..=5_000).contains(&retry_after_ms(&seen.body)));
    let seen = app.get(&format!("/rooms/{code}"), Some(&bob.token)).await;
    assert_eq!(seen.body["last_actor"], "alice");
    assert_eq!(retry_after_ms(&seen.body), 0);

    // The refused shot was not stored, and the delay runs out by itself.
    sqlx::query("UPDATE room SET last_action_at = now() - interval '5 s'")
        .execute(&app.pool)
        .await
        .unwrap();
    app.fire(&alice, 2).await.assert_status(StatusCode::CREATED);
}

#[sqlx::test]
async fn the_same_action_passes_once_the_opponent_acted(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, alice, bob) = app.playing_duel().await;

    app.fire(&alice, 1).await.assert_status(StatusCode::CREATED);
    app.fire(&bob, 1).await.assert_status(StatusCode::CREATED);

    app.fire(&alice, 2).await.assert_status(StatusCode::CREATED);
    app.fire(&alice, 3)
        .await
        .assert_status(StatusCode::CONFLICT);
}

#[sqlx::test]
async fn a_replayed_shot_neither_counts_nor_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    let (code, alice, bob) = app.playing_duel().await;
    app.fire(&alice, 1).await.assert_status(StatusCode::CREATED);

    // Within bob's priority, an entry already fired still answers its stored shot.
    app.fire(&alice, 1).await.assert_status(StatusCode::OK);
    app.fire(&bob, 1).await.assert_status(StatusCode::CREATED);
    app.fire(&alice, 20).await.assert_status(StatusCode::OK);

    let seen = app.get(&format!("/rooms/{code}"), Some(&alice.token)).await;
    assert_eq!(seen.body["last_actor"], "bob");
    app.fire(&alice, 2).await.assert_status(StatusCode::CREATED);
}

#[sqlx::test]
async fn a_race_has_no_priority(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, alice) = app.seed_with_token("alice", Role::User).await;
    let (_, bob) = app.seed_with_token("bob", Role::User).await;
    let created = app
        .post("/rooms", Some(&alice), json!({ "mode": "race" }))
        .await;
    let code = created.body["code"].as_str().unwrap();
    app.post(&format!("/rooms/{code}/join"), Some(&bob), json!({}))
        .await;
    let alice = Player {
        game: app.game_in(code, &alice).await,
        token: alice,
    };

    for entry in 1..=3 {
        app.fire(&alice, entry)
            .await
            .assert_status(StatusCode::CREATED);
    }
    let seen = app.get(&format!("/rooms/{code}"), Some(&alice.token)).await;
    assert!(seen.body.get("retry_after_ms").is_none(), "{}", seen.body);
}
