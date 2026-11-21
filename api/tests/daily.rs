#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `/daily/*` and `/ranking` end to end: the day's layout, one game per player, no surrender, and
//! the rankings by score and by points.

mod common;

use chrono::NaiveDate;
use common::{Response, TestApp};
use http::StatusCode;
use scout_beam::engine::Layout;
use scout_beam::services::daily_service::FIRST_DAY;
use scout_beam::shared::role::Role;
use serde_json::{Value, json};
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

    /// Started `offset` seconds past noon, Paris time, on its daily's day, solved `seconds` later.
    async fn time_game(&self, game_id: Uuid, offset: f64, seconds: f64) {
        sqlx::query(
            "UPDATE game
             SET created_at = ((daily_date + time '12:00') AT TIME ZONE 'Europe/Paris')
                              + make_interval(secs => $2),
                 solved_at = ((daily_date + time '12:00') AT TIME ZONE 'Europe/Paris')
                             + make_interval(secs => $2 + $3)
             WHERE id = $1",
        )
        .bind(game_id)
        .bind(offset)
        .bind(seconds)
        .execute(&self.pool)
        .await
        .unwrap();
    }

    /// A new account that solved that day's daily in `seconds`, started `offset` past noon.
    async fn solved_daily(
        &self,
        login: &str,
        date: NaiveDate,
        offset: f64,
        seconds: f64,
    ) -> String {
        let (_, token) = self.seed_with_token(login, Role::User).await;
        let game_id = self.start_daily(date, &token).await;
        self.solve(game_id, &token).await;
        self.time_game(game_id, offset, seconds).await;
        token
    }

    async fn insert_daily(&self, date: NaiveDate) {
        sqlx::query("INSERT INTO daily (date, layout) VALUES ($1, $2)")
            .bind(date)
            .bind(serde_json::to_value(layout_with_square("red")).unwrap())
            .execute(&self.pool)
            .await
            .unwrap();
    }

    /// A solved game of that day's daily, written straight to the table: past days before
    /// `FIRST_DAY` cannot be started through the API.
    async fn insert_daily_game(&self, user_id: Uuid, date: NaiveDate) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO game (user_id, layout, daily_date, status, created_at, solved_at)
             SELECT $1, layout, date, 'solved', now(), now() FROM daily WHERE date = $2
             RETURNING id",
        )
        .bind(user_id)
        .bind(date)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }

    async fn day_ranking(&self, date: NaiveDate, token: Option<&str>) -> Response {
        let ranking = self.get(&format!("/daily/{date}/ranking"), token).await;
        ranking.assert_status(StatusCode::OK);
        ranking
    }

    async fn ranking(&self, token: Option<&str>) -> Response {
        let ranking = self.get("/ranking", token).await;
        ranking.assert_status(StatusCode::OK);
        ranking
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

    let again = app.post_daily(today, &token).await;

    again.assert_status(StatusCode::OK);
    assert_eq!(again.body["id"], game_id.to_string());
    assert_eq!(again.body["shots"].as_array().unwrap().len(), 36);
}

#[sqlx::test]
async fn a_started_daily_has_every_entry_fired_in_entry_order(pool: PgPool) {
    let app = TestApp::new(pool);
    let today = app.today().await;
    let (_, token) = app.seed_with_token("owner", Role::User).await;

    let created = app.post_daily(today, &token).await;

    created.assert_status(StatusCode::CREATED);
    let game_id: Uuid = created.body["id"].as_str().unwrap().parse().unwrap();
    let expected: Vec<i64> = (1..=36).collect();
    assert_eq!(entries(&created), expected);
    assert_eq!(entries(&app.game(game_id, &token).await), expected);
}

#[sqlx::test]
async fn an_unfinished_daily_with_a_few_shots_gets_the_rest_and_a_finished_one_none(pool: PgPool) {
    let app = TestApp::new(pool);
    let today = app.today().await;
    app.insert_daily(today).await;
    let (owner, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.insert_daily_game(owner, today).await;
    sqlx::query("UPDATE game SET status = 'in_progress', solved_at = NULL WHERE id = $1")
        .bind(game_id)
        .execute(&app.pool)
        .await
        .unwrap();
    for entry in [12, 3] {
        app.play(
            game_id,
            Some(&token),
            json!({ "type": "fire", "entry": entry }),
        )
        .await
        .assert_status(StatusCode::CREATED);
    }
    let (solver, solver_token) = app.seed_with_token("solver", Role::User).await;
    let solved = app.insert_daily_game(solver, today).await;

    let resumed = app.post_daily(today, &token).await;
    let finished = app.post_daily(today, &solver_token).await;

    resumed.assert_status(StatusCode::OK);
    let rest = (1..=36).filter(|entry| ![12, 3].contains(entry));
    let expected: Vec<i64> = [12, 3].into_iter().chain(rest).collect();
    assert_eq!(entries(&resumed), expected);
    finished.assert_status(StatusCode::OK);
    assert_eq!(finished.body["id"], solved.to_string());
    assert_eq!(finished.body["shots"], json!([]));
}

#[sqlx::test]
async fn the_daily_game_is_read_without_being_started(pool: PgPool) {
    let app = TestApp::new(pool);
    let today = app.today().await;
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let uri = format!("/daily/{today}/game");
    app.get(&uri, Some(&token))
        .await
        .assert_status(StatusCode::NOT_FOUND);
    app.get(&uri, None)
        .await
        .assert_status(StatusCode::NOT_FOUND);

    let game_id = app.start_daily(today, &token).await;
    let read = app.get(&uri, Some(&token)).await;

    read.assert_status(StatusCode::OK);
    assert_eq!(read.body["id"], game_id.to_string());
    app.get(&uri, None)
        .await
        .assert_status(StatusCode::NOT_FOUND);
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
    assert_eq!(games.body[0]["mode"], "daily");
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
async fn no_daily_game_before_the_first_day(pool: PgPool) {
    let app = TestApp::new(pool);
    let before = FIRST_DAY.pred_opt().unwrap();
    app.insert_daily(before).await;
    let (_, token) = app.seed_with_token("owner", Role::User).await;

    app.post_daily(before, &token)
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn a_past_daily_can_still_be_played(pool: PgPool) {
    let app = TestApp::new(pool);
    let yesterday = app.today().await.pred_opt().unwrap();
    if yesterday < FIRST_DAY {
        return; // No past day is playable yet on the first day.
    }
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
async fn a_daily_fires_every_entry_at_once_and_has_no_pace(pool: PgPool) {
    let app = TestApp::new(pool);
    let today = app.today().await;
    let (_, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_daily(today, &token).await;

    let game = app.game(game_id, &token).await;
    assert!(game.body.get("elapsed_ms").is_none(), "{}", game.body);
    assert!(game.body.get("proposition_in_ms").is_none());
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

fn entries(game: &Response) -> Vec<i64> {
    game.body["shots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|shot| shot["entry"].as_i64().unwrap())
        .collect()
}

/// One field of every line of the top.
fn column(ranking: &Response, field: &str) -> Vec<Value> {
    ranking.body["top"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row[field].clone())
        .collect()
}

// --- rankings -------------------------------------------------------------------

#[sqlx::test]
async fn equal_scores_share_a_rank_and_the_first_solved_comes_first(pool: PgPool) {
    let app = TestApp::new(pool);
    let today = app.today().await;
    app.solved_daily("alice", today, 0.0, 100.0).await;
    app.solved_daily("bob", today, -50.0, 100.0).await;
    app.solved_daily("carol", today, 0.0, 200.0).await;

    let day = app.day_ranking(today, None).await;
    let global = app.ranking(None).await;

    assert_eq!(
        column(&day, "login"),
        [json!("bob"), json!("alice"), json!("carol")]
    );
    assert_eq!(column(&day, "rank"), [json!(1), json!(1), json!(3)]);
    assert_eq!(
        column(&global, "login"),
        [json!("alice"), json!("bob"), json!("carol")]
    );
    assert_eq!(column(&global, "rank"), [json!(1), json!(1), json!(3)]);
    assert!(day.body["me"].is_null());
}

#[sqlx::test]
async fn a_guest_appears_in_no_ranking(pool: PgPool) {
    let app = TestApp::new(pool);
    let today = app.today().await;
    let (_, token) = app.guest().await;
    let game_id = app.start_daily(today, &token).await;
    app.solve(game_id, &token).await;

    let day = app.day_ranking(today, Some(&token)).await;
    let global = app.ranking(Some(&token)).await;

    assert_eq!(day.body["top"], json!([]));
    assert!(day.body["me"].is_null());
    assert_eq!(global.body["top"], json!([]));
    assert!(global.body["me"].is_null());
}

#[sqlx::test]
async fn a_daily_played_late_is_ranked_after_the_day_and_counts_in_points(pool: PgPool) {
    let app = TestApp::new(pool);
    let yesterday = app.today().await.pred_opt().unwrap();
    app.insert_daily(yesterday).await;
    app.solved_daily("ontime", yesterday, 0.0, 600.0).await;
    let (late, token) = app.seed_with_token("late", Role::User).await;
    app.insert_daily_game(late, yesterday).await;

    let day = app.day_ranking(yesterday, Some(&token)).await;

    assert_eq!(column(&day, "login"), [json!("ontime"), json!("late")]);
    assert_eq!(column(&day, "late"), [json!(false), json!(true)]);
    assert_eq!(day.body["me"]["rank"], 2);
    let global = app.ranking(Some(&token)).await;
    assert_eq!(global.body["me"]["login"], "late");
    assert_eq!(global.body["me"]["dailies_solved"], 1);
}

#[sqlx::test]
async fn a_wrong_proposition_adds_a_minute_and_points_follow_the_score(pool: PgPool) {
    let app = TestApp::new(pool);
    let today = app.today().await;
    let yesterday = today.pred_opt().unwrap();
    app.insert_daily(yesterday).await;
    let (owner, token) = app.seed_with_token("owner", Role::User).await;
    let game_id = app.start_daily(today, &token).await;
    app.hide_layout(game_id, &layout_with_square("red")).await;
    for colour in ["blue", "yellow"] {
        let wrong = app
            .propose(game_id, Some(&token), &layout_with_square(colour))
            .await;
        assert_eq!(wrong.body["correct"], false, "{}", wrong.body);
    }
    app.solve(game_id, &token).await;
    app.time_game(game_id, 0.0, 180.4).await;
    let instant_id = app.insert_daily_game(owner, yesterday).await;
    app.time_game(instant_id, 0.0, 0.0).await;

    let day = app.day_ranking(today, Some(&token)).await;
    let global = app.ranking(Some(&token)).await;

    assert_eq!(day.body["me"]["seconds"], 180);
    assert_eq!(day.body["me"]["wrong_propositions"], 2);
    assert_eq!(day.body["me"]["score"], 300);
    // round(1000 - 200 * ln(1 + score / 60)): 642 for a score of 300, 1000 for 0.
    assert_eq!(global.body["me"]["points"], 642 + 1000);
    assert_eq!(global.body["me"]["dailies_solved"], 2);
}

#[sqlx::test]
async fn the_caller_outside_the_top_still_gets_their_line(pool: PgPool) {
    let app = TestApp::new(pool);
    let today = app.today().await;
    let token = app.solved_daily("caller", today, 0.0, 1000.0).await;
    sqlx::query(
        r#"WITH players AS (
               INSERT INTO "user" (login) SELECT 'player' || i FROM generate_series(1, 100) i
               RETURNING id
           )
           INSERT INTO game (user_id, layout, daily_date, status, created_at, solved_at)
           SELECT p.id, d.layout, d.date, 'solved', n.noon, n.noon + interval '60 seconds'
           FROM players p, daily d,
                LATERAL (SELECT (d.date + time '12:00') AT TIME ZONE 'Europe/Paris' AS noon) n
           WHERE d.date = $1"#,
    )
    .bind(today)
    .execute(&app.pool)
    .await
    .unwrap();

    let day = app.day_ranking(today, Some(&token)).await;
    let global = app.ranking(Some(&token)).await;
    let anonymous = app.ranking(None).await;

    for ranking in [&day, &global] {
        let logins = column(ranking, "login");
        assert_eq!(logins.len(), 100);
        assert!(!logins.contains(&json!("caller")));
        assert_eq!(ranking.body["me"]["login"], "caller");
        assert_eq!(ranking.body["me"]["rank"], 101);
    }
    assert!(anonymous.body["me"].is_null());
}
