//! Connection pool and schema migration.

use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Open the pool. Fails at boot if the database is unreachable, rather than on the first request.
pub async fn connect(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect(database_url)
        .await
        .context("cannot connect to PostgreSQL")?;

    tracing::info!(
        max_connections = MAX_CONNECTIONS,
        acquire_timeout_s = ACQUIRE_TIMEOUT.as_secs(),
        "connection pool opened — caps queries in flight, not requests per second"
    );

    Ok(pool)
}

/// Measured on the POC this crate comes from: throughput peaks at one to two connections per core.
const MAX_CONNECTIONS: u32 = 35;

/// Long enough to ride out a spike, short enough that a saturated pool surfaces as an error.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// Apply every pending migration, embedded at compile time: the binary deploys the schema it expects.
pub async fn migrate(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("database migration failed")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[tokio::test]
    async fn a_malformed_url_fails_before_any_socket_is_opened() {
        let error = connect("not-a-database-url").await.unwrap_err();

        assert!(
            error.to_string().contains("cannot connect"),
            "unexpected error: {error}"
        );
    }
}
