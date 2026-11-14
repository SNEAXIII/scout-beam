//! Process configuration, read once at boot.
//!
//! Everything fallible about the environment fails here, loudly, before the server binds.

use std::fmt;
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

/// A value `Debug` never prints, so a logged `Config` leaks no secret.
#[derive(Clone)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Secret(..)")
    }
}

/// An OAuth app at Discord or Google. The endpoints are settings so tests can point them at a mock.
#[derive(Debug, Clone)]
pub struct OAuthProvider {
    pub client_id: String,
    pub client_secret: Secret,
    pub authorize_url: String,
    pub token_url: String,
    pub profile_url: String,
}

/// The pepper of `email_hash`, and its version, so a rotation can be caught up account by account.
#[derive(Debug, Clone)]
pub struct EmailPepper {
    pub secret: Secret,
    pub version: i16,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: Secret,
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
    pub bind_addr: SocketAddr,
    pub cors_allowed_origin: String,
    pub environment: Environment,
    pub discord: Option<OAuthProvider>,
    pub google: Option<OAuthProvider>,
    /// Always set when a provider is: `from_source` refuses to boot otherwise.
    pub email_pepper: Option<EmailPepper>,
}

const DISCORD_ENDPOINTS: [&str; 3] = [
    "https://discord.com/oauth2/authorize",
    "https://discord.com/api/oauth2/token",
    "https://discord.com/api/users/@me",
];

const GOOGLE_ENDPOINTS: [&str; 3] = [
    "https://accounts.google.com/o/oauth2/v2/auth",
    "https://oauth2.googleapis.com/token",
    "https://openidconnect.googleapis.com/v1/userinfo",
];

impl Config {
    /// Read the configuration from the process environment.
    pub fn from_env() -> Result<Self> {
        Self::from_source(|key| std::env::var(key).ok())
    }

    /// Same, from an arbitrary lookup: the process environment is shared by every test in a binary.
    fn from_source(lookup: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let get = |key: &str| lookup(key).filter(|value| !value.is_empty());

        let discord = oauth_provider(&get, "DISCORD", DISCORD_ENDPOINTS)?;
        let google = oauth_provider(&get, "GOOGLE", GOOGLE_ENDPOINTS)?;
        let email_pepper = email_pepper(&get)?;
        anyhow::ensure!(
            email_pepper.is_some() || (discord.is_none() && google.is_none()),
            "EMAIL_PEPPER and EMAIL_PEPPER_VERSION are required once a sign-in provider is enabled"
        );

        Ok(Self {
            database_url: required(&get, "DATABASE_URL")?,
            jwt_secret: Secret::new(required(&get, "JWT_SECRET")?),
            access_token_ttl: Duration::from_mins(parse_ttl(
                &get,
                "ACCESS_TOKEN_EXPIRE_MINUTES",
                30,
                MAX_ACCESS_TOKEN_MINUTES,
            )?),
            refresh_token_ttl: Duration::from_hours(
                24 * parse_ttl(
                    &get,
                    "REFRESH_TOKEN_EXPIRE_DAYS",
                    30,
                    MAX_REFRESH_TOKEN_DAYS,
                )?,
            ),
            bind_addr: get("BIND_ADDR")
                .unwrap_or_else(|| "0.0.0.0:8002".to_owned())
                .parse()
                .context("BIND_ADDR is not a valid socket address")?,
            cors_allowed_origin: get("CORS_ALLOWED_ORIGIN")
                .unwrap_or_else(|| "http://localhost:3001".to_owned()),
            environment: environment(get("ENV").as_deref())?,
            discord,
            google,
            email_pepper,
        })
    }

    /// Config for tests: `#[sqlx::test]` supplies the pool, so only the secret has to be stable.
    /// Public because the integration tests are a separate crate.
    pub fn for_tests() -> Self {
        Self {
            database_url: String::new(),
            jwt_secret: Secret::new("test-secret"),
            access_token_ttl: Duration::from_mins(30),
            refresh_token_ttl: Duration::from_hours(24),
            bind_addr: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            cors_allowed_origin: "http://localhost:3001".to_owned(),
            environment: Environment::Dev,
            discord: None,
            google: None,
            email_pepper: None,
        }
    }
}

/// A provider is enabled by its id and secret together; one without the other is a typo, refused.
fn oauth_provider(
    get: &impl Fn(&str) -> Option<String>,
    prefix: &str,
    [authorize_url, token_url, profile_url]: [&str; 3],
) -> Result<Option<OAuthProvider>> {
    let id_key = format!("{prefix}_CLIENT_ID");
    let secret_key = format!("{prefix}_CLIENT_SECRET");

    match (get(&id_key), get(&secret_key)) {
        (None, None) => Ok(None),
        (Some(client_id), Some(client_secret)) => Ok(Some(OAuthProvider {
            client_id,
            client_secret: Secret::new(client_secret),
            authorize_url: authorize_url.to_owned(),
            token_url: token_url.to_owned(),
            profile_url: profile_url.to_owned(),
        })),
        _ => anyhow::bail!("{id_key} and {secret_key} go together: set both or neither"),
    }
}

fn email_pepper(get: &impl Fn(&str) -> Option<String>) -> Result<Option<EmailPepper>> {
    let Some(secret) = get("EMAIL_PEPPER") else {
        return Ok(None);
    };
    let raw = required(get, "EMAIL_PEPPER_VERSION")?;
    let version: i16 = raw
        .parse()
        .with_context(|| format!("EMAIL_PEPPER_VERSION must be a small integer, got {raw:?}"))?;
    anyhow::ensure!(
        version >= 1,
        "EMAIL_PEPPER_VERSION must be 1 or more, got {version}"
    );

    Ok(Some(EmailPepper {
        secret: Secret::new(secret),
        version,
    }))
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

const MAX_ACCESS_TOKEN_MINUTES: u64 = 24 * 60;
const MAX_REFRESH_TOKEN_DAYS: u64 = 365;

/// A token lifetime, refused at boot when zero or beyond `max`.
fn parse_ttl(
    get: &impl Fn(&str) -> Option<String>,
    key: &str,
    default: u64,
    max: u64,
) -> Result<u64> {
    let value = get(key).map_or(Ok(default), |raw| {
        raw.parse()
            .with_context(|| format!("{key} must be a positive integer, got {raw:?}"))
    })?;
    anyhow::ensure!(
        (1..=max).contains(&value),
        "{key} must be between 1 and {max}, got {value}"
    );
    Ok(value)
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
    fn a_ttl_out_of_bounds_fails_at_boot() {
        for days in ["0", "366", "99999999999999999"] {
            let error = Config::from_source(|key| match key {
                "DATABASE_URL" => Some("postgres://localhost/db".to_owned()),
                "JWT_SECRET" => Some("secret".to_owned()),
                "ENV" => Some("dev".to_owned()),
                "REFRESH_TOKEN_EXPIRE_DAYS" => Some(days.to_owned()),
                _ => None,
            })
            .unwrap_err()
            .to_string();

            assert!(
                error.contains("REFRESH_TOKEN_EXPIRE_DAYS"),
                "{days}: {error}"
            );
        }
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

    const WITH_DISCORD: &[(&str, &str)] = &[
        ("DATABASE_URL", "postgres://localhost/db"),
        ("JWT_SECRET", "jwt-secret-value"),
        ("ENV", "dev"),
        ("DISCORD_CLIENT_ID", "discord-client"),
        ("DISCORD_CLIENT_SECRET", "discord-secret-value"),
        ("EMAIL_PEPPER", "pepper-value"),
        ("EMAIL_PEPPER_VERSION", "2"),
    ];

    #[test]
    fn no_provider_is_enabled_by_default() {
        let config = Config::from_source(source(MINIMAL)).unwrap();

        assert!(config.discord.is_none() && config.google.is_none());
        assert!(config.email_pepper.is_none());
    }

    #[test]
    fn a_provider_is_enabled_by_its_id_and_secret_with_the_real_endpoints() {
        let config = Config::from_source(source(WITH_DISCORD)).unwrap();

        let discord = config.discord.unwrap();
        assert_eq!(discord.client_id, "discord-client");
        assert_eq!(discord.token_url, "https://discord.com/api/oauth2/token");
        assert!(config.google.is_none());
        assert_eq!(config.email_pepper.unwrap().version, 2);
    }

    #[test]
    fn a_client_id_without_its_secret_fails_at_boot() {
        for (present, missing) in [
            ("DISCORD_CLIENT_ID", "DISCORD_CLIENT_SECRET"),
            ("GOOGLE_CLIENT_SECRET", "GOOGLE_CLIENT_ID"),
        ] {
            let error = Config::from_source(|key| match key {
                "DATABASE_URL" => Some("postgres://localhost/db".to_owned()),
                "JWT_SECRET" => Some("secret".to_owned()),
                "ENV" => Some("dev".to_owned()),
                "EMAIL_PEPPER" => Some("pepper".to_owned()),
                "EMAIL_PEPPER_VERSION" => Some("1".to_owned()),
                key if key == present => Some("value".to_owned()),
                _ => None,
            })
            .unwrap_err()
            .to_string();

            assert!(error.contains(missing), "{present} alone: {error}");
        }
    }

    #[test]
    fn a_provider_without_a_pepper_fails_at_boot() {
        let error = Config::from_source(source(&[
            ("DATABASE_URL", "postgres://localhost/db"),
            ("JWT_SECRET", "secret"),
            ("ENV", "dev"),
            ("GOOGLE_CLIENT_ID", "google-client"),
            ("GOOGLE_CLIENT_SECRET", "google-secret"),
        ]))
        .unwrap_err()
        .to_string();

        assert!(error.contains("EMAIL_PEPPER"), "unhelpful message: {error}");
    }

    #[test]
    fn a_pepper_without_its_version_fails_at_boot() {
        let error = Config::from_source(source(&[
            ("DATABASE_URL", "postgres://localhost/db"),
            ("JWT_SECRET", "secret"),
            ("ENV", "dev"),
            ("EMAIL_PEPPER", "pepper"),
        ]))
        .unwrap_err()
        .to_string();

        assert!(error.contains("EMAIL_PEPPER_VERSION"), "{error}");
    }

    #[test]
    fn debug_prints_no_secret() {
        let printed = format!("{:?}", Config::from_source(source(WITH_DISCORD)).unwrap());

        for secret in ["jwt-secret-value", "discord-secret-value", "pepper-value"] {
            assert!(!printed.contains(secret), "{secret} leaked: {printed}");
        }
        assert!(printed.contains("discord-client"), "{printed}");
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
