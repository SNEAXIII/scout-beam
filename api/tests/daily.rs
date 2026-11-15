#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `/daily/*` end to end: the day's layout, one game per player, no surrender.

mod common;

use chrono::NaiveDate;
use common::{Response, TestApp};
use http::StatusCode;
use scout_beam::engine::Layout;
use scout_beam::shared::role::Role;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

impl TestApp {
    async fn today(&self) -> NaiveDate {
        let today = self.get("/daily/today", None).await;
        today.assert_status(StatusCode::OK);
        today.body["date"].as_str().unwrap().parse().unwrap()
    }

    async fn post_daily(&self, date: NaiveDate, token: &str) -> Response {
        self.post(&format!("/daily/{date}/game"), Some(token), json!({}))
            .await
    }

    async fn start_daily(&self, date: NaiveDate, token: &str) -> Uuid {
        let created = self.post_daily(date, token).await;
        created.assert_status(StatusCode::CREATED);
        created.body["id"].as_str().unwrap().parse().unwrap()
    }

    async fn game(&self, game_id: Uuid, token: &str) -> Response {
        let game = self.get(&format!("/games/{game_id}"), Some(token)).await;
        game.assert_status(StatusCode::OK);
        game
    }
}

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

#[sqlx::test]
async fn two_players_find_the_same_layout_today(pool: PgPool) {
    let app = TestApp::new(pool);
    let today = app.today().await;
    let (_, alice) = app.seed_with_token("alice", Role::User).await;
    let (_, bob) = app.seed_with_token("bob", Role::User).await;

    let alice_game = app.start_daily(today, &alice).await;
    let bob_game = app.start_daily(today, &bob).await;
    app.solve(alice_game, &alice).await;
    app.solve(bob_game, &bob).await;

    let alice_view = app.game(alice_game, &alice).await;
    let bob_view = app.game(bob_game, &bob).await;
    assert_ne!(alice_game, bob_game);
    assert!(alice_view.body["layout"].is_object());
    assert_eq!(alice_view.body["layout"], bob_view.body["layout"]);
    assert_eq!(alice_view.body["daily_date"], today.to_string());
}

#[sqlx::test]
async fn a_second_post_answers_the_same_game(pool: PgPool) {
    let app = TestApp::new(pool);
    let today = app.today().await;
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_daily(today, &token).await;
    app.play(game_id, Some(&token), json!({ "type": "fire", "entry": 7 }))
        .await
        .assert_status(StatusCode::CREATED);

    let again = app.post_daily(today, &token).await;

    again.assert_status(StatusCode::OK);
    assert_eq!(again.body["id"], game_id.to_string());
    assert_eq!(again.body["shots"][0]["entry"], 7);
}

#[sqlx::test]
async fn a_guest_can_start_the_daily_and_it_is_listed(pool: PgPool) {
    let app = TestApp::new(pool);
    let today = app.today().await;
    let (_, token) = app.guest().await;

    let game_id = app.start_daily(today, &token).await;

    let games = app.get("/games", Some(&token)).await;
    games.assert_status(StatusCode::OK);
    assert_eq!(games.body[0]["id"], game_id.to_string());
}

#[sqlx::test]
async fn a_day_to_come_or_a_past_day_without_daily_is_not_found(pool: PgPool) {
    let app = TestApp::new(pool);
    let today = app.today().await;
    let (_, token) = app.seed_with_token("owner", Role::User).await;

    let tomorrow = app.post_daily(today.succ_opt().unwrap(), &token).await;
    let yesterday = app.post_daily(today.pred_opt().unwrap(), &token).await;

    tomorrow.assert_status(StatusCode::NOT_FOUND);
    yesterday.assert_status(StatusCode::NOT_FOUND);
    assert!(
        yesterday.detail().contains("quotidienne"),
        "{}",
        yesterday.detail()
    );
}

#[sqlx::test]
async fn a_past_daily_can_still_be_played(pool: PgPool) {
    let app = TestApp::new(pool);
    let yesterday = app.today().await.pred_opt().unwrap();
    let layout = layout_with_square("red");
    sqlx::query("INSERT INTO daily (date, layout) VALUES ($1, $2)")
        .bind(yesterday)
        .bind(serde_json::to_value(&layout).unwrap())
        .execute(&app.pool)
        .await
        .unwrap();
    let (_, token) = app.seed_with_token("owner", Role::User).await;

    let game_id = app.start_daily(yesterday, &token).await;

    assert_eq!(app.stored_layout(game_id).await, layout);
}

#[sqlx::test]
async fn a_daily_cannot_be_surrendered(pool: PgPool) {
    let app = TestApp::new(pool);
    let today = app.today().await;
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_daily(today, &token).await;

    let refused = app
        .play(game_id, Some(&token), json!({ "type": "surrender" }))
        .await;

    refused.assert_status(StatusCode::CONFLICT);
    assert!(
        refused.detail().contains("quotidienne"),
        "{}",
        refused.detail()
    );
    assert_eq!(
        app.game(game_id, &token).await.body["status"],
        "in_progress"
    );
}

#[sqlx::test]
async fn only_wrong_propositions_are_counted(pool: PgPool) {
    let app = TestApp::new(pool);
    let today = app.today().await;
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_daily(today, &token).await;
    let hidden = layout_with_square("red");
    app.hide_layout(game_id, &hidden).await;
    let wrong = layout_with_square("blue");

    for _ in 0..2 {
        let proposed = app.propose(game_id, Some(&token), &wrong).await;
        assert_eq!(proposed.body["correct"], false, "{}", proposed.body);
    }
    app.propose(game_id, Some(&token), &Layout { pieces: Vec::new() })
        .await
        .assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    app.solve(game_id, &token).await;

    let game = app.game(game_id, &token).await;
    assert_eq!(game.body["wrong_propositions"], 2);
    assert_eq!(game.body["status"], "solved");
}

#[sqlx::test]
async fn a_game_against_the_bot_has_no_daily_date(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;

    let created = app.post("/games", Some(&token), json!({})).await;

    created.assert_status(StatusCode::CREATED);
    assert!(created.body["daily_date"].is_null());
    assert_eq!(created.body["wrong_propositions"], 0);
}
