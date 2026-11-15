//! Argon2id hashing and checking, off the async runtime: each call costs tens of milliseconds.

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

/// A well-formed hash of nothing, with the default parameters: checking against it costs what a real check costs.
const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHRzYWx0c2FsdA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

pub async fn hash(password: String) -> anyhow::Result<String> {
    tokio::task::spawn_blocking(move || {
        let hash = Argon2::default().hash_password(password.as_bytes())?;
        Ok(hash.to_string())
    })
    .await?
}

/// `false` when there is no stored hash, but a check is spent all the same, so the timing does not tell an unknown login apart.
pub async fn matches(password: String, stored: Option<String>) -> anyhow::Result<bool> {
    tokio::task::spawn_blocking(move || {
        let known = stored.is_some();
        let stored = stored.as_deref().unwrap_or(DUMMY_HASH);
        let Ok(parsed) = PasswordHash::new(stored) else {
            return Ok(false);
        };
        let verified = Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok();
        Ok(known && verified)
    })
    .await?
}
