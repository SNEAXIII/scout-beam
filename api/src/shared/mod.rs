//! Cross-cutting building blocks: `shared` may be used by any domain, and knows none of them.

pub mod db;
pub mod error;
pub mod openapi;
pub mod role;
pub mod validation;
