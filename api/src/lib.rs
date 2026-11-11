//! Scout Beam API.
//!
//! Exposed as a library so the integration tests can build the exact router the binary serves.

pub mod app;
pub mod auth;
pub mod config;
pub mod domains;
pub mod engine;
pub mod shared;
pub mod state;

pub use app::build_app;
pub use config::Config;
pub use state::AppState;
