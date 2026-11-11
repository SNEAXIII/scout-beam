//! `/auth/*` — turning an account into tokens, and back.
//!
//! In the user domain because each route reads the `user` table; `auth` stays a pure mechanism.

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::{get, post};

use crate::auth::dto::{LoginResponse, RefreshTokenRequest};
use crate::domains::user::dto::{DevLoginRequest, UserProfile};
use crate::domains::user::extractor::CurrentUser;
use crate::domains::user::service;
use crate::shared::error::ApiResult;
#[allow(unused_imports)] // referenced by the utoipa annotations
use crate::shared::openapi::ErrorResponse;
use crate::shared::validation::ValidatedJson;
use crate::state::AppState;

pub fn router(dev_login_enabled: bool) -> Router<AppState> {
    let router = Router::new()
        .route("/session", get(read_session))
        .route("/refresh", post(refresh));

    // Minting a token from nothing but a login must not exist in a production router.
    if dev_login_enabled {
        router.route("/dev-login", post(dev_login))
    } else {
        router
    }
}

/// `GET /auth/session`
#[utoipa::path(
    get,
    path = "/auth/session",
    tag = "Auth",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "Profile of the caller", body = UserProfile),
        (status = 401, description = "Missing, invalid or expired token", body = ErrorResponse),
    ),
)]
pub(crate) async fn read_session(CurrentUser(user): CurrentUser) -> Json<UserProfile> {
    Json(user.into())
}

/// `POST /auth/refresh` — trade a refresh token for a fresh pair, once the account is re-checked.
#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "Auth",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "New token pair", body = LoginResponse),
        (status = 401, description = "Not a valid refresh token, or unknown account", body = ErrorResponse),
    ),
)]
pub(crate) async fn refresh(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<RefreshTokenRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let claims = state.jwt.decode_refresh(&body.refresh_token)?;
    let user = service::load_active(&state.pool, claims.subject()?).await?;

    Ok(Json(LoginResponse::bearer(
        state.jwt.access_token(user.id, user.role)?,
        state.jwt.refresh_token(user.id)?,
    )))
}

/// `POST /auth/dev-login` — the development sign-in.
#[utoipa::path(
    post,
    path = "/auth/dev-login",
    tag = "Auth",
    request_body = DevLoginRequest,
    responses(
        (status = 200, description = "Token pair for the account", body = LoginResponse),
        (status = 422, description = "Login does not pass validation", body = ErrorResponse),
    ),
)]
pub(crate) async fn dev_login(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<DevLoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let user = service::login_or_create(&state.pool, body.login.as_deref(), body.role).await?;

    Ok(Json(LoginResponse::bearer(
        state.jwt.access_token(user.id, user.role)?,
        state.jwt.refresh_token(user.id)?,
    )))
}
