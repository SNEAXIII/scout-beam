//! JWT minting and verification: HS256, claims `sub`, `type`, `exp`, plus `role` on access tokens.

use std::time::Duration;

use chrono::Utc;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::error::AuthError;
use crate::shared::role::Role;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: Uuid,
    pub role: Role,
    pub exp: i64,
}

/// Carries identity but no privilege.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: Uuid,
    pub exp: i64,
}

/// The `type` claim picks the variant, so one token cannot pass for the other.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Claims {
    Access(AccessClaims),
    Refresh(RefreshClaims),
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
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
            validation: Validation::new(Algorithm::HS256),
            access_ttl,
            refresh_ttl,
        }
    }

    pub fn access_token(&self, sub: Uuid, role: Role) -> Result<String, AuthError> {
        let exp = expires_in(self.access_ttl);
        self.encode(&Claims::Access(AccessClaims { sub, role, exp }))
    }

    pub fn refresh_token(&self, sub: Uuid) -> Result<String, AuthError> {
        let exp = expires_in(self.refresh_ttl);
        self.encode(&Claims::Refresh(RefreshClaims { sub, exp }))
    }

    pub fn decode_access(&self, token: &str) -> Result<AccessClaims, AuthError> {
        match self.decode(token)? {
            Claims::Access(claims) => Ok(claims),
            Claims::Refresh(_) => Err(AuthError::WrongTokenType),
        }
    }

    pub fn decode_refresh(&self, token: &str) -> Result<RefreshClaims, AuthError> {
        match self.decode(token)? {
            Claims::Refresh(claims) => Ok(claims),
            Claims::Access(_) => Err(AuthError::WrongTokenType),
        }
    }

    fn decode(&self, token: &str) -> Result<Claims, AuthError> {
        decode::<Claims>(token, &self.decoding, &self.validation)
            .map(|data| data.claims)
            .map_err(|error| match error.kind() {
                ErrorKind::ExpiredSignature => AuthError::ExpiredToken,
                _ => AuthError::InvalidToken,
            })
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
        let sub = Uuid::new_v4();
        let token = codec().access_token(sub, Role::Admin).unwrap();

        let claims = codec().decode_access(&token).unwrap();

        assert_eq!(claims.sub, sub);
        assert_eq!(claims.role, Role::Admin);
    }

    #[test]
    fn refresh_token_is_not_accepted_as_access() {
        let token = codec().refresh_token(Uuid::new_v4()).unwrap();

        assert!(matches!(
            codec().decode_access(&token),
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

        assert!(matches!(
            other.decode_access(&token),
            Err(AuthError::InvalidToken)
        ));
    }

    #[test]
    fn expired_token_is_reported_as_expired() {
        // Past jsonwebtoken's default 60s leeway.
        let exp = Utc::now().timestamp() - 120;
        let claims = Claims::Access(AccessClaims {
            sub: Uuid::new_v4(),
            role: Role::User,
            exp,
        });
        let token = codec().encode(&claims).unwrap();

        assert!(matches!(
            codec().decode_access(&token),
            Err(AuthError::ExpiredToken)
        ));
    }
}
