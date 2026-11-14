//! The email as a lookup key only: PBKDF2-HMAC-SHA256 with a server-side pepper, never the address.

use std::fmt::Write;

use sha2::Sha256;

const ITERATIONS: u32 = 200_000;

/// Hex of the trimmed, lowercased email; off the async runtime, since it costs real CPU.
pub async fn hash(email: String, pepper: String) -> anyhow::Result<String> {
    Ok(tokio::task::spawn_blocking(move || digest(&email, &pepper)).await?)
}

fn digest(email: &str, pepper: &str) -> String {
    let normalized = email.trim().to_lowercase();
    let mut key = [0_u8; 32];
    pbkdf2::pbkdf2_hmac::<Sha256>(
        normalized.as_bytes(),
        pepper.as_bytes(),
        ITERATIONS,
        &mut key,
    );

    key.iter().fold(String::with_capacity(64), |mut hex, byte| {
        let _ = write!(hex, "{byte:02x}");
        hex
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_and_surrounding_spaces_do_not_matter_but_the_pepper_does() {
        let reference = digest("player@example.com", "pepper");

        assert_eq!(digest("  Player@Example.COM ", "pepper"), reference);
        assert_ne!(digest("player@example.com", "other pepper"), reference);
        assert_eq!(reference.len(), 64);
        assert!(!reference.contains("player"));
    }
}
