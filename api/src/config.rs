//! Process configuration, read once at boot.
//!
//! Everything fallible about the environment fails here, loudly, before the server binds.

use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use anyhow::{Context, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Dev,
    Prod,
}

impl Environment {
    pub fn is_dev(self) -> bool {
        self == Self::Dev
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
    pub bind_addr: SocketAddr,
    pub cors_allowed_origin: String,
    pub environment: Environment,
}

impl Config {
    /// Read the configuration from the process environment.
    pub fn from_env() -> Result<Self> {
        Self::from_source(|key| std::env::var(key).ok())
    }

    /// Same, from an arbitrary lookup: the process environment is shared by every test in a binary.
    pub fn from_source(lookup: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let get = |key: &str| lookup(key).filter(|value| !value.is_empty());

        Ok(Self {
            database_url: required(&get, "DATABASE_URL")?,
            jwt_secret: required(&get, "JWT_SECRET")?,
            access_token_ttl: Duration::from_secs(
                60 * parse_or(&get, "ACCESS_TOKEN_EXPIRE_MINUTES", 30)?,
            ),
            refresh_token_ttl: Duration::from_secs(
                60 * 60 * 24 * parse_or(&get, "REFRESH_TOKEN_EXPIRE_DAYS", 30)?,
            ),
            bind_addr: get("BIND_ADDR")
                .unwrap_or_else(|| "0.0.0.0:8002".to_owned())
                .parse()
                .context("BIND_ADDR is not a valid socket address")?,
            cors_allowed_origin: get("CORS_ALLOWED_ORIGIN")
                .unwrap_or_else(|| "http://localhost:3001".to_owned()),
            environment: environment(get("ENV").as_deref())?,
        })
    }

    /// Config for tests: `#[sqlx::test]` supplies the pool, so only the secret has to be stable.
    /// Public because the integration tests are a separate crate.
    pub fn for_tests() -> Self {
        Self {
            database_url: String::new(),
            jwt_secret: "test-secret".to_owned(),
            access_token_ttl: Duration::from_mins(30),
            refresh_token_ttl: Duration::from_hours(24),
            bind_addr: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            cors_allowed_origin: "http://localhost:3001".to_owned(),
            environment: Environment::Dev,
        }
    }
}

/// Fails closed: the dev login is mounted only on an explicit `ENV=dev`.
fn environment(raw: Option<&str>) -> Result<Environment> {
    match raw {
        Some("dev") => Ok(Environment::Dev),
        Some("prod") => Ok(Environment::Prod),
        Some(other) => anyhow::bail!("ENV must be `dev` or `prod`, got {other:?}"),
        None => anyhow::bail!("missing environment variable ENV (`dev` or `prod`)"),
    }
}

fn required(get: &impl Fn(&str) -> Option<String>, key: &str) -> Result<String> {
    get(key).with_context(|| format!("missing environment variable {key}"))
}

fn parse_or(get: &impl Fn(&str) -> Option<String>, key: &str, default: u64) -> Result<u64> {
    get(key).map_or(Ok(default), |raw| {
        raw.parse()
            .with_context(|| format!("{key} must be a positive integer, got {raw:?}"))
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    /// A lookup over a fixed list, standing in for the environment.
    fn source(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        move |key| {
            pairs
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_owned())
        }
    }

    const MINIMAL: &[(&str, &str)] = &[
        ("DATABASE_URL", "postgres://localhost/db"),
        ("JWT_SECRET", "secret"),
        ("ENV", "dev"),
    ];

    #[test]
    fn the_three_required_settings_are_enough() {
        let config = Config::from_source(source(MINIMAL)).unwrap();

        assert_eq!(config.access_token_ttl, Duration::from_mins(30));
        assert_eq!(config.refresh_token_ttl, Duration::from_hours(720));
        assert_eq!(config.bind_addr.port(), 8002);
        assert_eq!(config.cors_allowed_origin, "http://localhost:3001");
        assert_eq!(config.environment, Environment::Dev);
    }

    #[test]
    fn a_missing_secret_is_reported_by_name() {
        let error = Config::from_source(source(&[
            ("DATABASE_URL", "postgres://localhost/db"),
            ("ENV", "dev"),
        ]))
        .unwrap_err()
        .to_string();

        assert!(error.contains("JWT_SECRET"), "unhelpful message: {error}");
    }

    #[test]
    fn an_empty_value_counts_as_absent() {
        let error = Config::from_source(source(&[
            ("DATABASE_URL", "postgres://localhost/db"),
            ("JWT_SECRET", ""),
            ("ENV", "dev"),
        ]))
        .unwrap_err()
        .to_string();

        assert!(error.contains("JWT_SECRET"));
    }

    #[test]
    fn a_non_numeric_ttl_fails_at_boot() {
        let error = Config::from_source(source(&[
            ("DATABASE_URL", "postgres://localhost/db"),
            ("JWT_SECRET", "secret"),
            ("ENV", "dev"),
            ("ACCESS_TOKEN_EXPIRE_MINUTES", "half an hour"),
        ]))
        .unwrap_err()
        .to_string();

        assert!(error.contains("ACCESS_TOKEN_EXPIRE_MINUTES"));
    }

    #[test]
    fn a_bad_bind_address_fails_at_boot() {
        let error = Config::from_source(source(&[
            ("DATABASE_URL", "postgres://localhost/db"),
            ("JWT_SECRET", "secret"),
            ("ENV", "dev"),
            ("BIND_ADDR", "8002"),
        ]))
        .unwrap_err()
        .to_string();

        assert!(error.contains("BIND_ADDR"));
    }

    #[test]
    fn dev_and_prod_are_the_only_environments() {
        assert_eq!(environment(Some("dev")).unwrap(), Environment::Dev);
        assert_eq!(environment(Some("prod")).unwrap(), Environment::Prod);
        assert!(Environment::Dev.is_dev());
        assert!(!Environment::Prod.is_dev());
    }

    #[test]
    fn an_unknown_environment_fails_at_boot() {
        for value in ["staging", "Prod", "production", "DEV"] {
            let error = environment(Some(value)).unwrap_err().to_string();

            assert!(error.contains("ENV"), "{value} was accepted: {error}");
        }
    }

    #[test]
    fn an_unset_environment_fails_at_boot() {
        let error = Config::from_source(source(&[
            ("DATABASE_URL", "postgres://localhost/db"),
            ("JWT_SECRET", "secret"),
        ]))
        .unwrap_err()
        .to_string();

        assert!(error.contains("ENV"), "unhelpful message: {error}");
    }

    #[test]
    fn custom_values_are_honoured() {
        let config = Config::from_source(source(&[
            ("DATABASE_URL", "postgres://localhost/db"),
            ("JWT_SECRET", "secret"),
            ("ENV", "prod"),
            ("ACCESS_TOKEN_EXPIRE_MINUTES", "5"),
            ("REFRESH_TOKEN_EXPIRE_DAYS", "1"),
            ("BIND_ADDR", "127.0.0.1:9000"),
            ("CORS_ALLOWED_ORIGIN", "https://scout-beam.app"),
        ]))
        .unwrap();

        assert_eq!(config.access_token_ttl, Duration::from_mins(5));
        assert_eq!(config.refresh_token_ttl, Duration::from_hours(24));
        assert_eq!(config.bind_addr.to_string(), "127.0.0.1:9000");
        assert_eq!(config.cors_allowed_origin, "https://scout-beam.app");
    }
}
