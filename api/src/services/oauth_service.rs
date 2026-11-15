//! Discord and Google sign-in: trade a code for a profile, then find, link or create the account.
//!
//! The provider's username and avatar are never read; the email is kept only as `email_hash`.

use std::time::Duration;

use anyhow::Context;
use reqwest::Url;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::{Config, OAuthProvider};
use crate::errors::user_error::{UserError, is_unique_violation};
use crate::models::user_model::User;
use crate::repositories::user_repository;
use crate::services::{email_hash, login_generator};

/// Token and profile together, leaving the email hash room under the 10 s request timeout:
/// a slow provider answers the generic 401, never a 408.
const PROVIDER_EXCHANGE_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Discord,
    Google,
}

impl Provider {
    pub const ALL: [Self; 2] = [Self::Discord, Self::Google];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Discord => "discord",
            Self::Google => "google",
        }
    }

    const fn scope(self) -> &'static str {
        match self {
            Self::Discord => "identify email",
            Self::Google => "openid email",
        }
    }

    const fn settings(self, config: &Config) -> Option<&OAuthProvider> {
        match self {
            Self::Discord => config.discord.as_ref(),
            Self::Google => config.google.as_ref(),
        }
    }

    /// The id in this provider's column, `None` in the other: the repository's two slots.
    const fn columns(self, id: &str) -> (Option<&str>, Option<&str>) {
        match self {
            Self::Discord => (Some(id), None),
            Self::Google => (None, Some(id)),
        }
    }

    fn linked_id(self, user: &User) -> Option<&str> {
        match self {
            Self::Discord => user.discord_id.as_deref(),
            Self::Google => user.google_id.as_deref(),
        }
    }
}

/// Built from the configured front origin, never from the request.
fn redirect_uri(config: &Config, provider: Provider) -> String {
    let origin = config.cors_allowed_origin.trim_end_matches('/');
    format!("{origin}/login/callback/{}", provider.name())
}

/// Each enabled provider with the URL the front sends the browser to; the front appends `state`.
pub fn authorize_links(config: &Config) -> anyhow::Result<Vec<(&'static str, String)>> {
    Provider::ALL
        .into_iter()
        .filter_map(|provider| {
            provider
                .settings(config)
                .map(|settings| (provider, settings))
        })
        .map(|(provider, settings)| {
            let url = Url::parse_with_params(
                &settings.authorize_url,
                [
                    ("client_id", settings.client_id.as_str()),
                    ("redirect_uri", &redirect_uri(config, provider)),
                    ("response_type", "code"),
                    ("scope", provider.scope()),
                ],
            )
            .with_context(|| format!("{} authorize URL is invalid", provider.name()))?;
            Ok((provider.name(), url.into()))
        })
        .collect()
}

/// Sign in with a provider's code. A guest `caller` becomes the account when no other one matches.
pub async fn sign_in(
    pool: &PgPool,
    http: &reqwest::Client,
    config: &Config,
    provider: Provider,
    code: &str,
    caller: Option<Uuid>,
) -> Result<User, UserError> {
    let settings = provider
        .settings(config)
        .ok_or(UserError::ProviderDisabled)?;
    let pepper = config
        .email_pepper
        .as_ref()
        .context("a sign-in provider is enabled without EMAIL_PEPPER")?;

    let exchange = fetch_profile(http, config, provider, settings, code);
    let profile = tokio::time::timeout(PROVIDER_EXCHANGE_TIMEOUT, exchange)
        .await
        .unwrap_or_else(|elapsed| Err(anyhow::Error::new(elapsed).context("provider exchange")))
        .map_err(|error| {
            tracing::warn!(
                provider = provider.name(),
                error = format!("{error:#}"),
                "sign-in provider refused"
            );
            UserError::ProviderRefused
        })?;

    // An unverified email is never hashed: linking on it would hand the account to whoever typed it.
    let email_hash = match profile.verified_email {
        Some(email) => Some(email_hash::hash(email, pepper.secret.expose().to_owned()).await?),
        None => None,
    };

    let user = resolve(
        pool,
        provider,
        &profile.id,
        email_hash.as_deref(),
        pepper.version,
        caller,
    )
    .await?;
    Ok(user_repository::touch_last_login(pool, user.id).await?)
}

/// Provider id first, then verified email, then the caller's guest, then a new account.
/// A write that loses a race on a unique column means the row now exists: read again, once.
async fn resolve(
    pool: &PgPool,
    provider: Provider,
    provider_id: &str,
    email_hash: Option<&str>,
    pepper_version: i16,
    caller: Option<Uuid>,
) -> Result<User, UserError> {
    let (discord_id, google_id) = provider.columns(provider_id);
    let email_hash_version = email_hash.map(|_| pepper_version);

    for _ in 0..2 {
        if let Some(user) =
            user_repository::find_by_provider_id(pool, discord_id, google_id).await?
        {
            return refresh_email_hash(pool, user, email_hash, pepper_version).await;
        }

        if let Some(hash) = email_hash
            && let Some(user) = user_repository::find_by_email_hash(pool, hash).await?
        {
            match provider.linked_id(&user) {
                // Linked to this very id by a sign-in that committed since the lookup above.
                Some(linked) if linked == provider_id => {
                    return refresh_email_hash(pool, user, email_hash, pepper_version).await;
                }
                Some(_) => return Err(UserError::AlreadyLinked),
                None => {}
            }
            match user_repository::link_provider(pool, user.id, discord_id, google_id).await {
                Ok(Some(linked)) => return Ok(linked),
                // The slot filled meanwhile, or this id was taken by another account: read again.
                Ok(None) => continue,
                Err(error) if is_unique_violation(&error) => continue,
                Err(error) => return Err(error.into()),
            }
        }

        let converted = match caller {
            Some(guest_id) => user_repository::convert_guest_with_provider(
                pool,
                guest_id,
                discord_id,
                google_id,
                email_hash,
                email_hash_version,
            )
            .await
            .transpose(),
            None => None,
        };
        let written = if let Some(written) = converted {
            written
        } else {
            let login =
                login_generator::unique_login(pool, &login_generator::random_base()).await?;
            user_repository::insert_with_provider(
                pool,
                &login,
                discord_id,
                google_id,
                email_hash,
                email_hash_version,
            )
            .await
        };

        match written {
            Ok(user) => return Ok(user),
            Err(error) if is_unique_violation(&error) => {}
            Err(error) => return Err(error.into()),
        }
    }

    Err(anyhow::anyhow!(
        "the account kept changing during a {} sign-in",
        provider.name()
    )
    .into())
}

/// Catch a stale `email_hash` up with the current pepper; never adds a hash the account lacks.
async fn refresh_email_hash(
    pool: &PgPool,
    user: User,
    email_hash: Option<&str>,
    pepper_version: i16,
) -> Result<User, UserError> {
    let stale = user.email_hash.is_some() && user.email_hash_version != Some(pepper_version);
    let (true, Some(hash)) = (stale, email_hash) else {
        return Ok(user);
    };

    match user_repository::set_email_hash(pool, user.id, hash, pepper_version).await {
        Ok(updated) => Ok(updated),
        // Another account already holds this email under the current pepper: keep the old hash.
        Err(error) if is_unique_violation(&error) => Ok(user),
        Err(error) => Err(error.into()),
    }
}

/// What a provider says about the player, reduced to what is kept: the id, and a verified email.
struct Profile {
    id: String,
    verified_email: Option<String>,
}

#[derive(Deserialize)]
struct TokenAnswer {
    access_token: String,
}

#[derive(Deserialize)]
struct DiscordProfile {
    id: String,
    email: Option<String>,
    #[serde(default)]
    verified: bool,
}

#[derive(Deserialize)]
struct GoogleProfile {
    sub: String,
    email: Option<String>,
    #[serde(default)]
    email_verified: bool,
}

async fn fetch_profile(
    http: &reqwest::Client,
    config: &Config,
    provider: Provider,
    settings: &OAuthProvider,
    code: &str,
) -> anyhow::Result<Profile> {
    let redirect_uri = redirect_uri(config, provider);
    let token = http
        .post(&settings.token_url)
        .form(&[
            ("client_id", settings.client_id.as_str()),
            ("client_secret", settings.client_secret.expose()),
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &redirect_uri),
        ])
        .send()
        .await
        .context("token request")?;
    let token: TokenAnswer = successful(token)
        .await?
        .json()
        .await
        .context("token answer")?;

    let answer = http
        .get(&settings.profile_url)
        .bearer_auth(&token.access_token)
        .send()
        .await
        .context("profile request")?;
    let answer = successful(answer).await?;

    Ok(match provider {
        Provider::Discord => {
            let profile: DiscordProfile = answer.json().await.context("profile answer")?;
            Profile {
                id: profile.id,
                verified_email: profile.email.filter(|_| profile.verified),
            }
        }
        Provider::Google => {
            let profile: GoogleProfile = answer.json().await.context("profile answer")?;
            Profile {
                id: profile.sub,
                verified_email: profile.email.filter(|_| profile.email_verified),
            }
        }
    })
}

/// A refusal carries the provider's own words (an error code, never the profile) into the log.
async fn successful(response: reqwest::Response) -> anyhow::Result<reqwest::Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let url = response.url().clone();
    let body = response.text().await.unwrap_or_default();
    anyhow::bail!("{url} answered {status}: {body}")
}
