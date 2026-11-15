#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! A Duel end to end: each player hides a layout for the other, then each hunts at their own pace.

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

    /// A playing Duel hosted by alice, joined by bob.
    async fn playing_duel(&self) -> (Player, Player) {
        let (code, alice, bob) = self.composing_duel().await;
        self.hide(&code, &alice, &hidden()).await;
        let started = self.hide(&code, &bob, &hidden()).await;
        started.assert_status(StatusCode::CREATED);
        (
            Player {
                game: self.game_in(&code, &alice).await,
                token: alice,
            },
            Player {
                game: self.game_in(&code, &bob).await,
                token: bob,
            },
        )
    }

    /// Moves the game's start, and so its earned shots, that many milliseconds back.
    async fn start_earlier(&self, player: &Player, ms: i64) {
        sqlx::query(
            "UPDATE game SET created_at = created_at - $2 * interval '1 millisecond' WHERE id = $1",
        )
        .bind(player.game)
        .bind(ms)
        .execute(&self.pool)
        .await
        .unwrap();
    }

    /// Moves the player's propositions that many milliseconds back.
    async fn propose_earlier(&self, player: &Player, ms: i64) {
        sqlx::query(
            "UPDATE proposition SET created_at = created_at - $2 * interval '1 millisecond'
             WHERE game_id = $1",
        )
        .bind(player.game)
        .bind(ms)
        .execute(&self.pool)
        .await
        .unwrap();
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

    async fn propose_wrong(&self, player: &Player) -> Response {
        self.propose(player.game, Some(&player.token), &hidden())
            .await
    }

    async fn pace(&self, player: &Player) -> Value {
        self.get(&format!("/games/{}", player.game), Some(&player.token))
            .await
            .body
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
    assert_eq!(refused.code(), "missing_colour", "{}", refused.body);
    assert_eq!(refused.body["params"], json!({ "colour": "yellow" }));
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
    let (code, alice, bob) = app.composing_duel().await;
    let (_, stranger) = app.seed_with_token("stranger", Role::User).await;
    let (_, guest) = app.guest().await;
    let race = app
        .post("/rooms", Some(&alice), json!({ "mode": "race" }))
        .await;
    let race = race.body["code"].as_str().unwrap();

    let layout = hidden();
    let first = app.hide(&code, &alice, &layout).await;
    first.assert_status(StatusCode::CREATED);
    assert_eq!(first.body["hidden_layout"], json!(layout));
    let theirs = app.get(&format!("/rooms/{code}"), Some(&bob)).await;
    assert!(
        !theirs.body.to_string().contains("pieces"),
        "{}",
        theirs.body
    );
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
async fn fifteen_shots_go_at_once_then_the_next_waits_for_the_first_earned_one(pool: PgPool) {
    let app = TestApp::new(pool);
    let (alice, bob) = app.playing_duel().await;

    for entry in 1..=15 {
        app.fire(&alice, entry)
            .await
            .assert_status(StatusCode::CREATED);
    }
    let refused = app.fire(&alice, 16).await;

    refused.assert_status(StatusCode::CONFLICT);
    let left = retry_after_ms(&refused.body);
    assert!((1..=5_000).contains(&left), "{left}");
    // A replayed shot costs nothing, and the opponent's allowance is their own.
    app.fire(&alice, 1).await.assert_status(StatusCode::OK);
    app.fire(&bob, 1).await.assert_status(StatusCode::CREATED);

    app.start_earlier(&alice, 5_000).await;
    app.fire(&alice, 16)
        .await
        .assert_status(StatusCode::CREATED);
    let refused = app.fire(&alice, 17).await;
    refused.assert_status(StatusCode::CONFLICT);
    let left = retry_after_ms(&refused.body);
    assert!((1..=5_500).contains(&left), "{left}");
}

#[sqlx::test]
async fn two_tabs_firing_at_once_cannot_both_spend_the_last_shot(pool: PgPool) {
    let app = TestApp::new(pool);
    let (alice, _) = app.playing_duel().await;
    for entry in 1..=14 {
        app.fire(&alice, entry)
            .await
            .assert_status(StatusCode::CREATED);
    }
    // A slow build could earn a shot while the 14 are fired: restart the clock.
    sqlx::query("UPDATE game SET created_at = now() WHERE id = $1")
        .bind(alice.game)
        .execute(&app.pool)
        .await
        .unwrap();

    let (first, second) = tokio::join!(app.fire(&alice, 15), app.fire(&alice, 16));

    let mut statuses = [first.status, second.status];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::CREATED, StatusCode::CONFLICT]);
}

#[sqlx::test]
async fn earned_shots_accumulate_until_fired(pool: PgPool) {
    let app = TestApp::new(pool);
    let (alice, _) = app.playing_duel().await;
    // Earned at 5 s, 10.5 s and 16.5 s.
    app.start_earlier(&alice, 16_500).await;

    for entry in 1..=18 {
        app.fire(&alice, entry)
            .await
            .assert_status(StatusCode::CREATED);
    }
    app.fire(&alice, 19)
        .await
        .assert_status(StatusCode::CONFLICT);

    let game = app.pace(&alice).await;
    assert!(game["elapsed_ms"].as_u64().unwrap() >= 16_500, "{game}");
    assert_eq!(game["proposition_in_ms"], 0);
}

#[sqlx::test]
async fn the_proposition_cooldown_grows_with_wrong_propositions_up_to_10_s(pool: PgPool) {
    let app = TestApp::new(pool);
    let (alice, _) = app.playing_duel().await;

    let first = app.propose_wrong(&alice).await;
    first.assert_status(StatusCode::CREATED);
    assert_eq!(first.body["correct"], false, "{}", first.body);
    assert_eq!(first.body["proposition_in_ms"], 5_000);
    let refused = app.propose_wrong(&alice).await;
    refused.assert_status(StatusCode::CONFLICT);
    let left = retry_after_ms(&refused.body);
    assert!((4_000..=5_000).contains(&left), "{left}");

    app.propose_earlier(&alice, 5_000).await;
    let second = app.propose_wrong(&alice).await;
    second.assert_status(StatusCode::CREATED);
    assert_eq!(second.body["proposition_in_ms"], 6_000);

    // 5 s + 5 × 1 s reaches the cap at the 6th; the 7th stays there.
    for _ in 0..5 {
        app.propose_earlier(&alice, 10_000).await;
        app.propose_wrong(&alice)
            .await
            .assert_status(StatusCode::CREATED);
    }
    let capped = app.pace(&alice).await;
    assert_eq!(capped["wrong_propositions"], 7);
    let left = capped["proposition_in_ms"].as_u64().unwrap();
    assert!((9_000..=10_000).contains(&left), "{left}");
}

#[sqlx::test]
async fn a_race_has_the_same_pace(pool: PgPool) {
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

    for entry in 1..=15 {
        app.fire(&alice, entry)
            .await
            .assert_status(StatusCode::CREATED);
    }
    app.fire(&alice, 16)
        .await
        .assert_status(StatusCode::CONFLICT);
    app.propose_wrong(&alice)
        .await
        .assert_status(StatusCode::CREATED);
    app.propose_wrong(&alice)
        .await
        .assert_status(StatusCode::CONFLICT);
}
