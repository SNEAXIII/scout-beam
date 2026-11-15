//! Ranking rows, as the ranking queries compute them.

/// One player's solved daily of a day.
#[derive(Debug, Clone)]
pub struct DayRankingRow {
    pub rank: i64,
    pub login: String,
    pub score: i64,
    pub seconds: i64,
    pub wrong_propositions: i64,
    /// Within the first 100, ties cut by `solved_at` then login.
    pub in_top: bool,
    pub mine: bool,
}

/// One player's points over every daily they solved.
#[derive(Debug, Clone)]
pub struct GlobalRankingRow {
    pub rank: i64,
    pub login: String,
    pub points: i64,
    pub dailies_solved: i64,
    pub in_top: bool,
    pub mine: bool,
}
