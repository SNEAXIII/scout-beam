//! Making up a login when nobody supplies one: a themed word pair, collisions settled with digits.

use rand::RngExt;
use sqlx::PgPool;

use crate::domains::user::repository;

const ADJECTIVES: [&str; 10] = [
    "cosmic", "mighty", "iron", "shadow", "storm", "silver", "golden", "dark", "thunder", "mystic",
];

const NOUNS: [&str; 10] = [
    "hero", "blade", "hunter", "striker", "guardian", "avenger", "champion", "seeker", "warrior",
    "knight",
];

/// How many collisions we tolerate before giving up on a readable name.
const ATTEMPTS: u8 = 10;
const MAX_LENGTH: usize = 15;

/// `cosmicblade42` — an adjective, a noun and two digits.
pub fn random_base() -> String {
    let mut rng = rand::rng();
    // Indexing a constant, non-empty array: no Option to unwrap, no panic path.
    let adjective = ADJECTIVES[rng.random_range(0..ADJECTIVES.len())];
    let noun = NOUNS[rng.random_range(0..NOUNS.len())];
    let digits: u8 = rng.random_range(10..100);

    format!("{adjective}{noun}{digits}")
}

/// Strip anything the `login` column would reject, and pad a too-short result.
pub fn normalize(candidate: &str) -> String {
    let mut normalized: String = candidate
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect();

    if normalized.len() < 4 {
        let mut rng = rand::rng();
        let suffix: u16 = rng.random_range(1000..10_000);
        normalized = format!("{normalized}{suffix}");
    }

    normalized.truncate(MAX_LENGTH);
    normalized
}

/// A login no account holds yet. Racy by construction: the unique index on `login` is the
/// real guarantee, this only keeps the generated names pleasant.
pub async fn unique_login(pool: &PgPool, candidate: &str) -> sqlx::Result<String> {
    let base = normalize(candidate);
    let mut login = base.clone();

    for _ in 0..ATTEMPTS {
        if repository::find_by_login(pool, &login).await?.is_none() {
            return Ok(login);
        }

        let suffix: u16 = rand::rng().random_range(100..1000);
        let stem: String = base.chars().take(12).collect();
        login = format!("{stem}{suffix}");
    }

    // Ten collisions in a row: fall back to something that cannot realistically clash.
    let mut rng = rand::rng();
    let noise: String = (0..10)
        .map(|_| {
            let alphabet = b"abcdefghijklmnopqrstuvwxyz0123456789";
            alphabet[rng.random_range(0..alphabet.len())] as char
        })
        .collect();
    Ok(format!("user{noise}"))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn a_generated_login_passes_the_column_constraints() {
        for _ in 0..50 {
            let login = random_base();

            assert!((3..=30).contains(&login.len()), "bad length: {login}");
            assert!(
                login.chars().all(|c| c.is_ascii_alphanumeric()),
                "bad charset: {login}"
            );
        }
    }

    #[test]
    fn normalize_drops_what_the_column_forbids() {
        assert_eq!(normalize("Sne axi-iii"), "Sneaxiiii");
        assert_eq!(normalize("Snéaxiii"), "Snaxiii");
    }

    #[test]
    fn normalize_pads_a_short_name_and_caps_a_long_one() {
        let padded = normalize("ab");
        assert!(padded.len() >= 4, "not padded: {padded}");
        assert!(padded.starts_with("ab"));

        assert_eq!(normalize(&"a".repeat(40)).len(), MAX_LENGTH);
    }
}
