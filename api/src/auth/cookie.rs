//! The session cookies: the only way a token leaves the API, out of reach of the front's scripts.

use std::time::Duration;

use axum::http::header::COOKIE;
use axum::http::{HeaderMap, HeaderValue};
use uuid::Uuid;

use crate::auth::jwt::JwtCodec;
use crate::shared::error::ApiError;
use crate::shared::role::Role;

pub const ACCESS_COOKIE: &str = "scout_beam_access";
pub const REFRESH_COOKIE: &str = "scout_beam_refresh";

/// `Set-Cookie` values for a fresh token pair. `secure` is off only for plain-HTTP dev.
pub fn issue(
    codec: &JwtCodec,
    user_id: Uuid,
    role: Role,
    secure: bool,
) -> Result<[HeaderValue; 2], ApiError> {
    Ok([
        set_cookie(
            ACCESS_COOKIE,
            &codec.access_token(user_id, role)?,
            codec.access_ttl(),
            secure,
        )?,
        set_cookie(
            REFRESH_COOKIE,
            &codec.refresh_token(user_id)?,
            codec.refresh_ttl(),
            secure,
        )?,
    ])
}

/// `Set-Cookie` values that make the browser drop both cookies.
pub fn clear(secure: bool) -> Result<[HeaderValue; 2], ApiError> {
    Ok([
        set_cookie(ACCESS_COOKIE, "", Duration::ZERO, secure)?,
        set_cookie(REFRESH_COOKIE, "", Duration::ZERO, secure)?,
    ])
}

fn set_cookie(
    name: &str,
    value: &str,
    max_age: Duration,
    secure: bool,
) -> Result<HeaderValue, ApiError> {
    let secure = if secure { "; Secure" } else { "" };
    let cookie = format!(
        "{name}={value}; HttpOnly; SameSite=Strict; Path=/; Max-Age={}{secure}",
        max_age.as_secs()
    );
    HeaderValue::try_from(cookie).map_err(|error| ApiError::Internal(error.into()))
}

/// A cookie's value out of the request's `Cookie` headers.
pub fn read<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|header| header.to_str().ok())
        .flat_map(|header| header.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, value)| *key == name && !value.is_empty())
        .map(|(_, value)| value)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn a_cookie_is_http_only_strict_and_secure_outside_dev() {
        let codec = JwtCodec::new("secret", Duration::from_mins(30), Duration::from_hours(24));

        let [access, refresh] = issue(&codec, Uuid::nil(), Role::User, true).unwrap();
        let access = access.to_str().unwrap();

        assert!(access.starts_with("scout_beam_access="), "{access}");
        for part in [
            "HttpOnly",
            "SameSite=Strict",
            "Path=/",
            "Max-Age=1800",
            "Secure",
        ] {
            assert!(access.contains(part), "{part} missing from {access}");
        }
        assert!(refresh.to_str().unwrap().contains("Max-Age=86400"));

        let [dev, _] = issue(&codec, Uuid::nil(), Role::User, false).unwrap();
        assert!(!dev.to_str().unwrap().contains("Secure"));
    }

    #[test]
    fn a_cookie_is_read_among_others() {
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_static("theme=dark; scout_beam_access=abc.def; other=1"),
        );

        assert_eq!(read(&headers, ACCESS_COOKIE), Some("abc.def"));
        assert_eq!(read(&headers, REFRESH_COOKIE), None);
    }
}
