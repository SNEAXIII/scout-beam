//! JWT minting and verification: HS256, claims `user_id`, `role`, `type`, `exp`.

use std::time::Duration;

use chrono::Utc;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::error::AuthError;
use crate::shared::role::Role;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Stringified UUID.
    pub user_id: String,

    /// Absent on refresh tokens, which carry identity but no privilege.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,

    #[serde(rename = "type")]
    pub token_type: TokenType,

    pub exp: i64,
}

impl Claims {
    pub fn subject(&self) -> Result<Uuid, AuthError> {
        Uuid::parse_str(&self.user_id).map_err(|_| AuthError::MissingSubject)
    }

    /// The role an access token must carry; refuses refresh tokens.
    pub fn access_role(&self) -> Result<Role, AuthError> {
        match self.token_type {
            TokenType::Refresh => Err(AuthError::WrongTokenType),
            TokenType::Access => self.role.ok_or(AuthError::InvalidRole),
        }
    }
}

/// Owns the signing material, built once and shared through `AppState`.
#[derive(Clone)]
pub struct JwtCodec {
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
    access_ttl: Duration,
    refresh_ttl: Duration,
}

impl std::fmt::Debug for JwtCodec {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never let the key reach a log line.
        formatter.debug_struct("JwtCodec").finish_non_exhaustive()
    }
}

impl JwtCodec {
    pub fn new(secret: &str, access_ttl: Duration, refresh_ttl: Duration) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        // No `aud`/`iss` is minted, so none is required.
        validation.required_spec_claims.clear();

        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
            validation,
            access_ttl,
            refresh_ttl,
        }
    }

    pub fn access_token(&self, user_id: Uuid, role: Role) -> Result<String, AuthError> {
        self.encode(&Claims {
            user_id: user_id.to_string(),
            role: Some(role),
            token_type: TokenType::Access,
            exp: expires_in(self.access_ttl),
        })
    }

    pub fn refresh_token(&self, user_id: Uuid) -> Result<String, AuthError> {
        self.encode(&Claims {
            user_id: user_id.to_string(),
            role: None,
            token_type: TokenType::Refresh,
            exp: expires_in(self.refresh_ttl),
        })
    }

    pub fn decode(&self, token: &str) -> Result<Claims, AuthError> {
        let data =
            decode::<Claims>(token, &self.decoding, &self.validation).map_err(
                |error| match error.kind() {
                    ErrorKind::ExpiredSignature => AuthError::ExpiredToken,
                    _ => AuthError::InvalidToken,
                },
            )?;

        if data.claims.user_id.is_empty() {
            return Err(AuthError::MissingSubject);
        }
        Ok(data.claims)
    }

    /// Decode a refresh token, refusing an access token used in its place.
    pub fn decode_refresh(&self, token: &str) -> Result<Claims, AuthError> {
        let claims = self.decode(token)?;
        match claims.token_type {
            TokenType::Refresh => Ok(claims),
            TokenType::Access => Err(AuthError::WrongTokenType),
        }
    }

    fn encode(&self, claims: &Claims) -> Result<String, AuthError> {
        encode(&Header::new(Algorithm::HS256), claims, &self.encoding)
            .map_err(|_| AuthError::InvalidToken)
    }
}

fn expires_in(ttl: Duration) -> i64 {
    // Saturating beats wrapping into a past timestamp, which would mint an expired token.
    let seconds = i64::try_from(ttl.as_secs()).unwrap_or(i64::MAX);
    Utc::now().timestamp().saturating_add(seconds)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    fn codec() -> JwtCodec {
        JwtCodec::new(
            "test-secret",
            Duration::from_mins(1),
            Duration::from_hours(1),
        )
    }

    #[test]
    fn access_token_round_trips() {
        let user_id = Uuid::new_v4();
        let token = codec().access_token(user_id, Role::Admin).unwrap();

        let claims = codec().decode(&token).unwrap();

        assert_eq!(claims.subject().unwrap(), user_id);
        assert_eq!(claims.access_role().unwrap(), Role::Admin);
    }

    #[test]
    fn refresh_token_carries_no_role() {
        let token = codec().refresh_token(Uuid::new_v4()).unwrap();

        let claims = codec().decode(&token).unwrap();

        assert!(claims.role.is_none());
        assert!(matches!(
            claims.access_role(),
            Err(AuthError::WrongTokenType)
        ));
    }

    #[test]
    fn access_token_is_not_accepted_as_refresh() {
        let token = codec().access_token(Uuid::new_v4(), Role::User).unwrap();

        assert!(matches!(
            codec().decode_refresh(&token),
            Err(AuthError::WrongTokenType)
        ));
    }

    #[test]
    fn another_secret_is_rejected() {
        let token = codec().access_token(Uuid::new_v4(), Role::User).unwrap();
        let other = JwtCodec::new("other", Duration::from_mins(1), Duration::from_mins(1));

        assert!(matches!(other.decode(&token), Err(AuthError::InvalidToken)));
    }

    #[test]
    fn expired_token_is_reported_as_expired() {
        let expired = JwtCodec::new(
            "test-secret",
            Duration::from_secs(0),
            Duration::from_secs(0),
        );
        let token = expired.access_token(Uuid::new_v4(), Role::User).unwrap();

        // jsonwebtoken allows 60s of clock skew by default; without disabling it a zero TTL validates.
        let mut strict = codec();
        strict.validation.leeway = 0;
        std::thread::sleep(Duration::from_millis(1100));

        assert!(matches!(
            strict.decode(&token),
            Err(AuthError::ExpiredToken)
        ));
    }
}
