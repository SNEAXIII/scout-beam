//! `/auth/*` — turning an account into tokens, and back.
//!
//! Here rather than in `auth`: each route reads the `user` table, and `auth` stays a pure mechanism.

use axum::Json;
use axum::extract::State;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::auth::dto::{LoginResponse, RefreshTokenRequest};
use crate::dto::user_dto::{DevLoginRequest, UserProfile};
use crate::extractors::current_user_extractor::CurrentUser;
use crate::services::user_service;
use crate::shared::error::ApiResult;
use crate::shared::openapi::ErrorResponse;
use crate::shared::validation::ValidatedJson;
use crate::state::AppState;

pub fn router(dev_login_enabled: bool) -> OpenApiRouter<AppState> {
    let router = OpenApiRouter::new()
        .routes(routes!(get_session))
        .routes(routes!(post_refresh_token));

    // Minting a token from nothing but a login must not exist in a production router.
    if dev_login_enabled {
        router.routes(routes!(post_dev_login))
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
async fn get_session(CurrentUser(user): CurrentUser) -> Json<UserProfile> {
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
async fn post_refresh_token(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<RefreshTokenRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let claims = state.jwt.decode_refresh(&body.refresh_token)?;
    let user = user_service::load_active(&state.pool, claims.sub).await?;

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
async fn post_dev_login(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<DevLoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let user = user_service::login_or_create(&state.pool, body.login.as_deref(), body.role).await?;

    Ok(Json(LoginResponse::bearer(
        state.jwt.access_token(user.id, user.role)?,
        state.jwt.refresh_token(user.id)?,
    )))
}
