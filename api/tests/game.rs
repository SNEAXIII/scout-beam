#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `/games` end to end: starting a game, reading it back, firing shots.

mod common;

use common::{Response, TestApp};
use http::StatusCode;
use scout_beam::engine::{self, Entry, Layout};
use scout_beam::shared::role::Role;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

impl TestApp {
    async fn fire(&self, game_id: Uuid, token: &str, entry: Value) -> Response {
        self.post(
            &format!("/games/{game_id}/shots"),
            Some(token),
            json!({ "entry": entry }),
        )
        .await
    }

    async fn shot_count(&self, game_id: Uuid) -> i64 {
        sqlx::query_scalar("SELECT count(*) FROM shot WHERE game_id = $1")
            .bind(game_id)
            .fetch_one(&self.pool)
            .await
            .expect("shots are counted")
    }
}

// --- POST /games --------------------------------------------------------------

#[sqlx::test]
async fn a_new_game_is_in_progress_with_no_shot_and_no_layout(pool: PgPool) {
    let app = TestApp::new(pool);
    let (user_id, token) = app.seed_with_token("owner", Role::User).await;

    let created = app.post("/games", Some(&token), json!({})).await;

    created.assert_status(StatusCode::CREATED);
    assert_eq!(created.body["status"], "in_progress");
    assert_eq!(created.body["shots"], json!([]));
    assert!(created.body["solved_at"].is_null());
    assert!(created.body.get("layout").is_none(), "{}", created.body);

    let id: Uuid = created.body["id"].as_str().unwrap().parse().unwrap();
    let owner: Uuid = sqlx::query_scalar("SELECT user_id FROM game WHERE id = $1")
        .bind(id)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(owner, user_id);
    app.stored_layout(id)
        .await
        .validate()
        .expect("the bot draws a valid layout");
}

#[sqlx::test]
async fn an_anonymous_caller_cannot_start_a_game(pool: PgPool) {
    let app = TestApp::new(pool);

    app.post("/games", None, json!({}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);

    let games: i64 = sqlx::query_scalar("SELECT count(*) FROM game")
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(games, 0);
}

// --- GET /games ---------------------------------------------------------------

#[sqlx::test]
async fn the_game_list_holds_only_the_callers_games_newest_first_without_layout(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let (_, other_token) = app.seed_with_token("other", Role::User).await;
    let first = app.start_game(&token).await;
    app.start_game(&other_token).await;
    let second = app.start_game(&token).await;

    let listed = app.get("/games", Some(&token)).await;

    listed.assert_status(StatusCode::OK);
    let games = listed.body.as_array().expect("a list");
    let ids: Vec<&str> = games
        .iter()
        .map(|game| game["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, [second.to_string(), first.to_string()]);
    for game in games {
        assert!(game.get("layout").is_none(), "{game}");
        assert_eq!(game["status"], "in_progress");
    }
}

#[sqlx::test]
async fn an_anonymous_caller_cannot_list_games(pool: PgPool) {
    let app = TestApp::new(pool);

    app.get("/games", None)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

// --- GET /games/{id} ----------------------------------------------------------

#[sqlx::test]
async fn a_game_reads_back_its_shots_in_firing_order_without_layout(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;
    for entry in [12, 3, 30] {
        app.fire(game_id, &token, json!(entry))
            .await
            .assert_status(StatusCode::CREATED);
    }

    let read = app.get(&format!("/games/{game_id}"), Some(&token)).await;

    read.assert_status(StatusCode::OK);
    assert_eq!(read.body["id"], game_id.to_string());
    assert_eq!(read.body["status"], "in_progress");
    let entries: Vec<i64> = read.body["shots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|shot| shot["entry"].as_i64().unwrap())
        .collect();
    assert_eq!(entries, [12, 3, 30]);
    assert!(read.body.get("layout").is_none(), "{}", read.body);
}

#[sqlx::test]
async fn a_solved_game_reveals_its_layout(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;
    app.solve(game_id, &token).await;

    let read = app.get(&format!("/games/{game_id}"), Some(&token)).await;

    read.assert_status(StatusCode::OK);
    assert_eq!(read.body["status"], "solved");
    assert!(read.body["solved_at"].is_string());
    let revealed: Layout = serde_json::from_value(read.body["layout"].clone()).unwrap();
    assert_eq!(revealed, app.stored_layout(game_id).await);
}

#[sqlx::test]
async fn another_players_game_is_not_found(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, owner_token) = app.seed_with_token("owner", Role::User).await;
    let (_, intruder_token) = app.seed_with_token("intruder", Role::User).await;
    let game_id = app.start_game(&owner_token).await;

    let read = app
        .get(&format!("/games/{game_id}"), Some(&intruder_token))
        .await;

    read.assert_status(StatusCode::NOT_FOUND);
    assert!(read.body.get("layout").is_none());
}

#[sqlx::test]
async fn an_unknown_game_is_not_found(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;

    app.get(&format!("/games/{}", Uuid::new_v4()), Some(&token))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn an_anonymous_caller_cannot_read_a_game(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;

    app.get(&format!("/games/{game_id}"), None)
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}

// --- POST /games/{id}/shots ---------------------------------------------------

#[sqlx::test]
async fn a_shot_answers_what_the_engine_traces_on_the_hidden_layout(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;

    let fired = app.fire(game_id, &token, json!(7)).await;

    fired.assert_status(StatusCode::CREATED);
    let expected =
        engine::trace(&app.stored_layout(game_id).await, Entry::new(7).unwrap()).unwrap();
    assert_eq!(
        fired.body,
        json!({
            "entry": 7,
            "exit_entry": expected.exit.number(),
            "colour": expected.beam_colour,
        })
    );
    assert_eq!(app.shot_count(game_id).await, 1);
}

#[sqlx::test]
async fn re_firing_an_entry_answers_the_stored_shot_without_a_second_row(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;
    let first = app.fire(game_id, &token, json!(18)).await;
    first.assert_status(StatusCode::CREATED);

    let again = app.fire(game_id, &token, json!(18)).await;

    again.assert_status(StatusCode::OK);
    assert_eq!(again.body, first.body);
    assert_eq!(app.shot_count(game_id).await, 1);
}

#[sqlx::test]
async fn concurrent_re_fires_of_one_entry_store_a_single_shot(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;

    let (left, right) = tokio::join!(
        app.fire(game_id, &token, json!(25)),
        app.fire(game_id, &token, json!(25)),
    );

    let mut statuses = [left.status, right.status];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::OK, StatusCode::CREATED]);
    assert_eq!(left.body, right.body);
    assert_eq!(app.shot_count(game_id).await, 1);
}

#[sqlx::test]
async fn firing_on_a_solved_game_is_a_conflict(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;
    app.solve(game_id, &token).await;

    let fired = app.fire(game_id, &token, json!(7)).await;

    fired.assert_status(StatusCode::CONFLICT);
    assert!(
        fired.detail().contains("déjà résolue"),
        "{}",
        fired.detail()
    );
    assert_eq!(app.shot_count(game_id).await, 0);
}

#[sqlx::test]
async fn entries_outside_one_to_thirty_six_are_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;

    for entry in [json!(0), json!(37), json!(-1), json!(300), json!("7")] {
        let fired = app.fire(game_id, &token, entry.clone()).await;
        fired.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
        assert!(
            fired.body.get("exit_entry").is_none(),
            "{entry}: {}",
            fired.body
        );
    }
    let refused = app.fire(game_id, &token, json!(37)).await;
    assert!(
        refused.detail().contains("de 1 à 36"),
        "{}",
        refused.detail()
    );
    assert_eq!(app.shot_count(game_id).await, 0);
}

#[sqlx::test]
async fn the_first_and_last_entries_can_be_fired(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;

    for entry in [1, 36] {
        app.fire(game_id, &token, json!(entry))
            .await
            .assert_status(StatusCode::CREATED);
    }
    assert_eq!(app.shot_count(game_id).await, 2);
}

#[sqlx::test]
async fn another_players_game_cannot_be_fired_at(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, owner_token) = app.seed_with_token("owner", Role::User).await;
    let (_, intruder_token) = app.seed_with_token("intruder", Role::User).await;
    let game_id = app.start_game(&owner_token).await;

    app.fire(game_id, &intruder_token, json!(7))
        .await
        .assert_status(StatusCode::NOT_FOUND);

    assert_eq!(app.shot_count(game_id).await, 0);
}

#[sqlx::test]
async fn an_anonymous_caller_cannot_fire(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_game(&token).await;

    app.post(
        &format!("/games/{game_id}/shots"),
        None,
        json!({ "entry": 7 }),
    )
    .await
    .assert_status(StatusCode::UNAUTHORIZED);

    assert_eq!(app.shot_count(game_id).await, 0);
}
