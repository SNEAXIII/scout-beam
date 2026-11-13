#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `propose` on `/games/{id}/ws` end to end: checking a layout against the hidden one.

mod common;

use common::TestApp;
use http::StatusCode;
use scout_beam::domains::game::{GameError, service};
use scout_beam::engine::Layout;
use scout_beam::shared::role::Role;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

impl TestApp {
    /// `(correct, layout)` of each stored proposition, oldest first.
    async fn propositions(&self, game_id: Uuid) -> Vec<(bool, Layout)> {
        let rows: Vec<(bool, Value)> = sqlx::query_as(
            "SELECT correct, layout FROM proposition WHERE game_id = $1 ORDER BY created_at, id",
        )
        .bind(game_id)
        .fetch_all(&self.pool)
        .await
        .expect("propositions are read");
        rows.into_iter()
            .map(|(correct, layout)| (correct, serde_json::from_value(layout).unwrap()))
            .collect()
    }

    /// `(status, solved_at is set)` as stored.
    async fn game_state(&self, game_id: Uuid) -> (String, bool) {
        sqlx::query_as("SELECT status::text, solved_at IS NOT NULL FROM game WHERE id = $1")
            .bind(game_id)
            .fetch_one(&self.pool)
            .await
            .expect("the game is stored")
    }

    /// A game whose hidden layout is `layout` rather than the bot's draw.
    async fn game_hiding(&self, token: &str, layout: &Layout) -> Uuid {
        let game_id = self.start_game(token).await;
        self.hide_layout(game_id, layout).await;
        game_id
    }
}

/// The engine's own search case: the red square answers every entry alike at x 5 and x 7.
fn layout_with_square(x: u8, colour: &str) -> Layout {
    serde_json::from_value(json!({ "pieces": [
        { "shape": "square", "colour": colour, "x": x, "y": 0, "rotation": 0 },
        { "shape": "bar", "colour": "red", "x": 7, "y": 3, "rotation": 1 },
        { "shape": "spike", "colour": "white", "x": 5, "y": 4, "rotation": 0 },
        { "shape": "corner", "colour": "blue", "x": 3, "y": 2, "rotation": 0 },
        { "shape": "diamond", "colour": "yellow", "x": 1, "y": 5, "rotation": 0 },
    ]}))
    .unwrap()
}

fn hidden() -> Layout {
    layout_with_square(5, "red")
}

fn revealed(body: &Value) -> Layout {
    serde_json::from_value(body["layout"].clone()).expect("the layout is revealed")
}

// --- correct ------------------------------------------------------------------

#[sqlx::test]
async fn the_hidden_layout_itself_is_correct_and_solves_the_game(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;
    let layout = app.stored_layout(game_id).await;

    let proposed = app.propose(game_id, Some(&token), &layout).await;

    proposed.assert_status(StatusCode::CREATED);
    assert_eq!(proposed.body["correct"], true);
    assert_eq!(revealed(&proposed.body), layout);
    assert_eq!(app.game_state(game_id).await, ("solved".to_owned(), true));
    assert_eq!(app.propositions(game_id).await, [(true, layout)]);
}

#[sqlx::test]
async fn a_layout_placed_differently_but_indistinguishable_is_correct(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.game_hiding(&token, &hidden()).await;
    let moved = layout_with_square(7, "red");
    assert_ne!(moved, hidden());

    let proposed = app.propose(game_id, Some(&token), &moved).await;

    proposed.assert_status(StatusCode::CREATED);
    assert_eq!(proposed.body["correct"], true);
    assert_eq!(
        revealed(&proposed.body),
        hidden(),
        "the hidden one, not the proposition"
    );
    assert_eq!(app.propositions(game_id).await, [(true, moved)]);
}

#[sqlx::test]
async fn a_game_solved_by_a_proposition_reads_back_its_hidden_layout(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.game_hiding(&token, &hidden()).await;
    app.propose(game_id, Some(&token), &layout_with_square(7, "red"))
        .await
        .assert_status(StatusCode::CREATED);

    let read = app.get(&format!("/games/{game_id}"), Some(&token)).await;

    read.assert_status(StatusCode::OK);
    assert_eq!(read.body["status"], "solved");
    assert!(read.body["solved_at"].is_string());
    assert_eq!(revealed(&read.body), hidden());
}

// --- wrong --------------------------------------------------------------------

#[sqlx::test]
async fn a_wrong_proposition_is_stored_and_reveals_nothing(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.game_hiding(&token, &hidden()).await;
    let recoloured = layout_with_square(5, "blue");

    let proposed = app.propose(game_id, Some(&token), &recoloured).await;

    proposed.assert_status(StatusCode::CREATED);
    assert_eq!(proposed.body, json!({ "correct": false }));
    assert_eq!(
        app.game_state(game_id).await,
        ("in_progress".to_owned(), false)
    );
    assert_eq!(app.propositions(game_id).await, [(false, recoloured)]);
    let read = app.get(&format!("/games/{game_id}"), Some(&token)).await;
    assert!(read.body.get("layout").is_none(), "{}", read.body);
}

#[sqlx::test]
async fn a_correct_proposition_after_a_wrong_one_still_solves_the_game(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.game_hiding(&token, &hidden()).await;
    app.propose(game_id, Some(&token), &layout_with_square(5, "blue"))
        .await
        .assert_status(StatusCode::CREATED);

    let proposed = app.propose(game_id, Some(&token), &hidden()).await;

    assert_eq!(proposed.body["correct"], true);
    let stored: Vec<bool> = app
        .propositions(game_id)
        .await
        .into_iter()
        .map(|(correct, _)| correct)
        .collect();
    assert_eq!(stored, [false, true]);
}

// --- refused ------------------------------------------------------------------

#[sqlx::test]
async fn an_invalid_layout_is_refused_with_the_rule_it_breaks_and_not_stored(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.game_hiding(&token, &hidden()).await;

    let without_diamond = Layout {
        pieces: hidden().pieces[..4].to_vec(),
    };
    let mut two_squares = hidden();
    two_squares.pieces[4] = two_squares.pieces[0];
    two_squares.pieces[4].x = 9;
    let mut outside = hidden();
    outside.pieces[1].y = 7; // the bar, upright, needs rows 7 and 8
    let mut overlapping = hidden();
    overlapping.pieces[0].x = 7;
    overlapping.pieces[0].y = 3;
    let mut touching = hidden();
    touching.pieces[0].x = 7;
    touching.pieces[0].y = 2;

    for (layout, rule) in [
        (without_diamond, "Il manque la pièce « losange »"),
        (two_squares, "La pièce « carré » est placée plus d'une fois"),
        (outside, "La pièce « barre » dépasse de la grille"),
        (
            overlapping,
            "Les pièces « carré » et « barre » se chevauchent",
        ),
        (
            touching,
            "Les pièces « carré » et « barre » se touchent par un côté",
        ),
    ] {
        let proposed = app.propose(game_id, Some(&token), &layout).await;
        proposed.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
        assert!(
            proposed.detail().contains(rule),
            "{rule}: {}",
            proposed.detail()
        );
    }
    assert!(app.propositions(game_id).await.is_empty());
    assert_eq!(
        app.game_state(game_id).await,
        ("in_progress".to_owned(), false)
    );
}

#[sqlx::test]
async fn a_malformed_piece_is_refused_and_not_stored(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;
    let piece = |shape: &str, rotation: i64| json!({ "type": "propose", "pieces": [{ "shape": shape, "colour": "red", "x": 0, "y": 0, "rotation": rotation }] });

    for message in [
        piece("triangle", 0),
        piece("square", 4),
        json!({ "type": "propose" }),
    ] {
        app.play(game_id, Some(&token), message)
            .await
            .assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    }
    assert!(app.propositions(game_id).await.is_empty());
}

#[sqlx::test]
async fn a_proposition_on_a_solved_game_is_a_conflict(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.game_hiding(&token, &hidden()).await;
    app.solve(game_id, &token).await;

    let proposed = app
        .propose(game_id, Some(&token), &layout_with_square(7, "red"))
        .await;

    proposed.assert_status(StatusCode::CONFLICT);
    assert!(
        proposed.detail().contains("déjà résolue"),
        "{}",
        proposed.detail()
    );
    assert!(proposed.body.get("layout").is_none());
    assert_eq!(app.propositions(game_id).await.len(), 1);
}

/// The losing side of a race, without timing: its game was read before the other one committed.
#[sqlx::test]
async fn a_proposition_on_a_game_solved_since_it_was_read_is_a_conflict(pool: PgPool) {
    let app = TestApp::new(pool);
    let (user_id, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.game_hiding(&token, &hidden()).await;
    let read_before = service::get_owned(&app.pool, game_id, user_id)
        .await
        .unwrap();
    app.solve(game_id, &token).await;

    let late = service::propose(&app.pool, &read_before, &layout_with_square(7, "red")).await;

    assert!(matches!(late, Err(GameError::AlreadySolved)), "{late:?}");
    let stored = app.propositions(game_id).await;
    assert_eq!(stored, [(true, hidden())]);
}

#[sqlx::test]
async fn another_players_game_cannot_receive_a_proposition(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, owner_token) = app.seed_with_token("owner", Role::User).await;
    let (_, intruder_token) = app.seed_with_token("intruder", Role::User).await;
    let game_id = app.game_hiding(&owner_token, &hidden()).await;

    let proposed = app.propose(game_id, Some(&intruder_token), &hidden()).await;

    proposed.assert_status(StatusCode::NOT_FOUND);
    assert!(proposed.body.get("layout").is_none());
    assert!(app.propositions(game_id).await.is_empty());
    assert_eq!(
        app.game_state(game_id).await,
        ("in_progress".to_owned(), false)
    );
}

#[sqlx::test]
async fn an_unknown_game_cannot_receive_a_proposition(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;

    app.propose(Uuid::new_v4(), Some(&token), &hidden())
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn an_anonymous_caller_cannot_propose(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.game_hiding(&token, &hidden()).await;

    app.propose(game_id, None, &hidden())
        .await
        .assert_status(StatusCode::UNAUTHORIZED);

    assert!(app.propositions(game_id).await.is_empty());
    assert_eq!(
        app.game_state(game_id).await,
        ("in_progress".to_owned(), false)
    );
}
