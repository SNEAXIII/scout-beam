//! Authentication mechanism: tokens in, identity out.
//!
//! A leaf: it touches no table and knows no domain, so any domain may depend on it.

pub mod dto;
pub mod error;
pub mod extractor;
pub mod jwt;

pub use error::AuthError;
pub use extractor::Identity;
pub use jwt::{Claims, JwtCodec, TokenType};
