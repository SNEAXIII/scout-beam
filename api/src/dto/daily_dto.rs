//! Wire shapes of the daily domain.

use chrono::NaiveDate;
use serde::Serialize;

use crate::models::daily_model::{DayRankingRow, GlobalRankingRow};
use crate::services::daily_service::Ranking;

/// The current daily's date, in Paris.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DailyTodayResponse {
    #[schema(example = "2026-11-15")]
    pub date: NaiveDate,
}

/// A player's line in a day's ranking.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DayRankingEntry {
    /// Equal scores share a rank.
    pub rank: i64,
    pub login: String,
    /// Seconds to solve, plus 60 per wrong proposition; lower is better.
    pub score: i64,
    /// Seconds between the start and the correct proposition.
    pub seconds: i64,
    pub wrong_propositions: i64,
    /// Started after its day: ranked after every player who played it on time.
    pub late: bool,
}

impl From<DayRankingRow> for DayRankingEntry {
    fn from(row: DayRankingRow) -> Self {
        Self {
            rank: row.rank,
            login: row.login,
            score: row.score,
            seconds: row.seconds,
            wrong_propositions: row.wrong_propositions,
            late: row.late,
        }
    }
}

/// The best 100 of a day, on time before late, by score then by who solved it first.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DayRankingResponse {
    pub top: Vec<DayRankingEntry>,
    /// The caller's own line, inside the top or not; null when signed out or unranked.
    pub me: Option<DayRankingEntry>,
}

impl From<Ranking<DayRankingRow>> for DayRankingResponse {
    fn from(ranking: Ranking<DayRankingRow>) -> Self {
        Self {
            top: ranking.top.into_iter().map(Into::into).collect(),
            me: ranking.me.map(Into::into),
        }
    }
}

/// A player's line in the global ranking.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GlobalRankingEntry {
    /// Equal points share a rank.
    pub rank: i64,
    pub login: String,
    pub points: i64,
    pub dailies_solved: i64,
}

impl From<GlobalRankingRow> for GlobalRankingEntry {
    fn from(row: GlobalRankingRow) -> Self {
        Self {
            rank: row.rank,
            login: row.login,
            points: row.points,
            dailies_solved: row.dailies_solved,
        }
    }
}

/// The best 100 by points summed over every solved daily, then by login.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GlobalRankingResponse {
    pub top: Vec<GlobalRankingEntry>,
    /// The caller's own line, inside the top or not; null when signed out or unranked.
    pub me: Option<GlobalRankingEntry>,
}

impl From<Ranking<GlobalRankingRow>> for GlobalRankingResponse {
    fn from(ranking: Ranking<GlobalRankingRow>) -> Self {
        Self {
            top: ranking.top.into_iter().map(Into::into).collect(),
            me: ranking.me.map(Into::into),
        }
    }
}
