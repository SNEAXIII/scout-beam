//! Wire shapes of the daily domain.

use chrono::NaiveDate;
use serde::Serialize;

/// The current daily's date, in Paris.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DailyTodayResponse {
    #[schema(example = "2026-11-15")]
    pub date: NaiveDate,
}
