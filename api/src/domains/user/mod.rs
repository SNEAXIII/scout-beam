//! User domain: an account that can log in.

pub mod dto;
pub mod error;
pub mod extractor;
pub mod login_generator;
pub mod model;
pub mod repository;
pub mod routes;
pub mod service;

pub use error::UserError;
pub use extractor::CurrentUser;
pub use model::User;

use axum::Router;
use utoipa::OpenApi;

use crate::state::AppState;

/// Mount points owned by this domain.
pub fn router(dev_login_enabled: bool) -> Router<AppState> {
    Router::new().nest("/auth", routes::session::router(dev_login_enabled))
}

/// The domain's slice of the OpenAPI document; `app.rs` only merges the pieces.
#[derive(OpenApi)]
#[openapi(
    paths(
        routes::session::dev_login,
        routes::session::read_session,
        routes::session::refresh,
    ),
    components(schemas(
        dto::UserProfile,
        dto::DevLoginRequest,
        crate::auth::dto::LoginResponse,
        crate::auth::dto::RefreshTokenRequest,
        crate::shared::role::Role,
        crate::shared::openapi::ErrorResponse,
    )),
    tags((name = "Auth", description = "Sessions and tokens")),
)]
pub struct UserApi;

pub fn openapi() -> utoipa::openapi::OpenApi {
    UserApi::openapi()
}
