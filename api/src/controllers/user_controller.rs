//! `/auth/*` — turning an account into tokens, and back.
//!
//! Here rather than in `auth`: each route reads the `user` table, and `auth` stays a pure mechanism.

use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use http::StatusCode;
use serde_json::json;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::auth::Identity;
use crate::auth::dto::{LoginResponse, RefreshTokenRequest};
use crate::dto::user_dto::{DevLoginRequest, LoginRequest, RegisterRequest, UserProfile};
use crate::extractors::current_user_extractor::CurrentUser;
use crate::services::user_service;
use crate::shared::error::ApiResult;
use crate::shared::openapi::ErrorResponse;
use crate::shared::validation::ValidatedJson;
use crate::state::AppState;

/// Five sign-in attempts a minute per address, shared by every route that signs in or creates an account.
#[allow(clippy::expect_used)] // A fixed quota: `finish` cannot return `None`.
fn sign_in_router() -> OpenApiRouter<AppState> {
    let config = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(12)
        .burst_size(5)
        .finish()
        .expect("the quota is valid");

    // Forget addresses whose quota is full again; the `Weak` ends the loop with its router.
    let limiter = Arc::downgrade(config.limiter());
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_mins(1));
        loop {
            tick.tick().await;
            let Some(limiter) = limiter.upgrade() else {
                break;
            };
            limiter.retain_recent();
        }
    });

    let limit = GovernorLayer::new(Arc::new(config)).error_handler(|_| {
        let body = json!({ "detail": "Trop de tentatives, réessayez dans une minute" });
        (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response()
    });

    OpenApiRouter::new()
        .routes(routes!(post_register))
        .routes(routes!(post_login))
        .routes(routes!(post_guest))
        .layer(limit)
}

pub fn router(dev_login_enabled: bool) -> OpenApiRouter<AppState> {
    let router = OpenApiRouter::new()
        .routes(routes!(get_session))
        .routes(routes!(post_refresh_token))
        .merge(sign_in_router());

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

/// `POST /auth/register` — a new password account, signed in at once; a guest's bearer converts that guest.
#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "Auth",
    security((), ("bearer" = [])),
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Account created, token pair", body = LoginResponse),
        (status = 400, description = "Password outside 8 to 128 characters", body = ErrorResponse),
        (status = 409, description = "Login already taken", body = ErrorResponse),
        (status = 422, description = "Login does not pass validation", body = ErrorResponse),
        (status = 429, description = "Too many sign-in attempts from this address", body = ErrorResponse),
    ),
)]
async fn post_register(
    State(state): State<AppState>,
    caller: Option<Identity>,
    ValidatedJson(body): ValidatedJson<RegisterRequest>,
) -> ApiResult<(StatusCode, Json<LoginResponse>)> {
    let caller = caller.map(|identity| identity.user_id);
    let user =
        user_service::register_with_password(&state.pool, &body.login, &body.password, caller)
            .await?;

    Ok((
        StatusCode::CREATED,
        Json(LoginResponse::bearer(
            state.jwt.access_token(user.id, user.role)?,
            state.jwt.refresh_token(user.id)?,
        )),
    ))
}

/// `POST /auth/login` — sign in with a login and password.
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Token pair for the account", body = LoginResponse),
        (status = 401, description = "Unknown login, no password, or wrong password", body = ErrorResponse),
        (status = 429, description = "Too many sign-in attempts from this address", body = ErrorResponse),
    ),
)]
async fn post_login(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    let user = user_service::login_with_password(&state.pool, &body.login, &body.password).await?;

    Ok(Json(LoginResponse::bearer(
        state.jwt.access_token(user.id, user.role)?,
        state.jwt.refresh_token(user.id)?,
    )))
}

/// `POST /auth/guest` — a guest account under a generated login, signed in at once.
#[utoipa::path(
    post,
    path = "/auth/guest",
    tag = "Auth",
    responses(
        (status = 201, description = "Guest created, token pair", body = LoginResponse),
        (status = 429, description = "Too many sign-in attempts from this address", body = ErrorResponse),
    ),
)]
async fn post_guest(State(state): State<AppState>) -> ApiResult<(StatusCode, Json<LoginResponse>)> {
    let user = user_service::create_guest(&state.pool).await?;

    Ok((
        StatusCode::CREATED,
        Json(LoginResponse::bearer(
            state.jwt.access_token(user.id, user.role)?,
            state.jwt.refresh_token(user.id)?,
        )),
    ))
}
