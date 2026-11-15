//! `/auth/*` — turning an account into session cookies, and back.
//!
//! Here rather than in `auth`: each route reads the `user` table, and `auth` stays a pure mechanism.

use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;
use axum::response::{AppendHeaders, IntoResponse, Response};
use http::StatusCode;
use serde_json::json;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::auth::cookie::{self, REFRESH_COOKIE};
use crate::auth::{AuthError, Identity};
use crate::dto::user_dto::{
    DevLoginRequest, LoginRequest, ProviderCodeRequest, ProviderLink, ProvidersResponse,
    RegisterRequest, UserProfile,
};
use crate::extractors::current_user_extractor::CurrentUser;
use crate::models::user_model::User;
use crate::services::oauth_service::{self, Provider};
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
        let body = json!({
            "code": "too_many_attempts",
            "detail": "Too many attempts, try again in a minute",
        });
        (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response()
    });

    OpenApiRouter::new()
        .routes(routes!(post_register))
        .routes(routes!(post_login))
        .routes(routes!(post_guest))
        .routes(routes!(post_discord))
        .routes(routes!(post_google))
        .layer(limit)
}

pub fn router(dev_login_enabled: bool) -> OpenApiRouter<AppState> {
    let router = OpenApiRouter::new()
        .routes(routes!(get_session))
        .routes(routes!(post_refresh_token))
        .routes(routes!(post_logout))
        .routes(routes!(get_providers))
        .merge(sign_in_router());

    // Minting a token from nothing but a login must not exist in a production router.
    if dev_login_enabled {
        router.routes(routes!(post_dev_login))
    } else {
        router
    }
}

/// `Secure` everywhere but plain-HTTP dev.
fn secure_cookies(state: &AppState) -> bool {
    !state.config.environment.is_dev()
}

/// Both tokens as `HttpOnly` cookies, the profile as the body: a body carrying the tokens would
/// let a script read a fresh pair by calling `/auth/refresh`.
fn session_response(state: &AppState, user: User, status: StatusCode) -> ApiResult<Response> {
    let [access, refresh] = cookie::issue(&state.jwt, user.id, user.role, secure_cookies(state))?;
    let cookies = AppendHeaders([(SET_COOKIE, access), (SET_COOKIE, refresh)]);
    Ok((status, cookies, Json(UserProfile::from(user))).into_response())
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

/// `POST /auth/refresh` — the refresh cookie traded for a fresh pair, once the account is re-checked.
#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "Auth",
    responses(
        (status = 200, description = "New session cookies; the body is the profile", body = UserProfile),
        (status = 401, description = "No valid refresh cookie, or unknown account", body = ErrorResponse),
    ),
)]
async fn post_refresh_token(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let token = cookie::read(&headers, REFRESH_COOKIE).ok_or(AuthError::MissingToken)?;
    let claims = state.jwt.decode_refresh(token)?;
    let user = user_service::load_active(&state.pool, claims.sub).await?;

    session_response(&state, user, StatusCode::OK)
}

/// `POST /auth/logout` — the browser drops both cookies; a token copied elsewhere stays valid until it expires.
#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "Auth",
    responses((status = 204, description = "Session cookies cleared")),
)]
async fn post_logout(State(state): State<AppState>) -> ApiResult<Response> {
    let [access, refresh] = cookie::clear(secure_cookies(&state))?;
    let cookies = AppendHeaders([(SET_COOKIE, access), (SET_COOKIE, refresh)]);
    Ok((StatusCode::NO_CONTENT, cookies).into_response())
}

/// `POST /auth/dev-login` — the development sign-in.
#[utoipa::path(
    post,
    path = "/auth/dev-login",
    tag = "Auth",
    request_body = DevLoginRequest,
    responses(
        (status = 200, description = "Signed in: session cookies, the profile as the body", body = UserProfile),
        (status = 422, description = "Login does not pass validation", body = ErrorResponse),
    ),
)]
async fn post_dev_login(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<DevLoginRequest>,
) -> ApiResult<Response> {
    let user = user_service::login_or_create(&state.pool, body.login.as_deref(), body.role).await?;
    session_response(&state, user, StatusCode::OK)
}

/// `POST /auth/register` — a new password account, signed in at once; a guest's token converts that guest.
#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "Auth",
    security((), ("bearer" = [])),
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Account created and signed in: session cookies, the profile as the body", body = UserProfile),
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
) -> ApiResult<Response> {
    let caller = caller.map(|identity| identity.user_id);
    let user =
        user_service::register_with_password(&state.pool, &body.login, &body.password, caller)
            .await?;
    session_response(&state, user, StatusCode::CREATED)
}

/// `POST /auth/login` — sign in with a login and password.
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Signed in: session cookies, the profile as the body", body = UserProfile),
        (status = 401, description = "Unknown login, no password, or wrong password", body = ErrorResponse),
        (status = 429, description = "Too many sign-in attempts from this address", body = ErrorResponse),
    ),
)]
async fn post_login(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<LoginRequest>,
) -> ApiResult<Response> {
    let user = user_service::login_with_password(&state.pool, &body.login, &body.password).await?;
    session_response(&state, user, StatusCode::OK)
}

/// `POST /auth/guest` — a guest account under a generated login, signed in at once.
#[utoipa::path(
    post,
    path = "/auth/guest",
    tag = "Auth",
    responses(
        (status = 201, description = "Guest created and signed in: session cookies, the profile as the body", body = UserProfile),
        (status = 429, description = "Too many sign-in attempts from this address", body = ErrorResponse),
    ),
)]
async fn post_guest(State(state): State<AppState>) -> ApiResult<Response> {
    let user = user_service::create_guest(&state.pool).await?;
    session_response(&state, user, StatusCode::CREATED)
}

/// `GET /auth/providers` — what the sign-in page may offer.
#[utoipa::path(
    get,
    path = "/auth/providers",
    tag = "Auth",
    responses((status = 200, description = "Enabled sign-in providers", body = ProvidersResponse)),
)]
async fn get_providers(State(state): State<AppState>) -> ApiResult<Json<ProvidersResponse>> {
    let providers = oauth_service::authorize_links(&state.config)?
        .into_iter()
        .map(|(name, authorize_url)| ProviderLink {
            name,
            authorize_url,
        })
        .collect();

    Ok(Json(ProvidersResponse {
        dev_login: state.config.environment.is_dev(),
        providers,
    }))
}

/// `POST /auth/discord` — sign in with the code Discord handed the front.
#[utoipa::path(
    post,
    path = "/auth/discord",
    tag = "Auth",
    security((), ("bearer" = [])),
    request_body = ProviderCodeRequest,
    responses(
        (status = 200, description = "Signed in: session cookies, the profile as the body", body = UserProfile),
        (status = 401, description = "The provider refused the code or could not be reached", body = ErrorResponse),
        (status = 404, description = "Discord sign-in is not configured", body = ErrorResponse),
        (status = 409, description = "The account with this email holds another Discord id", body = ErrorResponse),
        (status = 429, description = "Too many sign-in attempts from this address", body = ErrorResponse),
    ),
)]
async fn post_discord(
    State(state): State<AppState>,
    caller: Option<Identity>,
    ValidatedJson(body): ValidatedJson<ProviderCodeRequest>,
) -> ApiResult<Response> {
    provider_sign_in(&state, Provider::Discord, caller, &body.code).await
}

/// `POST /auth/google` — sign in with the code Google handed the front.
#[utoipa::path(
    post,
    path = "/auth/google",
    tag = "Auth",
    security((), ("bearer" = [])),
    request_body = ProviderCodeRequest,
    responses(
        (status = 200, description = "Signed in: session cookies, the profile as the body", body = UserProfile),
        (status = 401, description = "The provider refused the code or could not be reached", body = ErrorResponse),
        (status = 404, description = "Google sign-in is not configured", body = ErrorResponse),
        (status = 409, description = "The account with this email holds another Google id", body = ErrorResponse),
        (status = 429, description = "Too many sign-in attempts from this address", body = ErrorResponse),
    ),
)]
async fn post_google(
    State(state): State<AppState>,
    caller: Option<Identity>,
    ValidatedJson(body): ValidatedJson<ProviderCodeRequest>,
) -> ApiResult<Response> {
    provider_sign_in(&state, Provider::Google, caller, &body.code).await
}

/// A guest's token converts that guest when no account matches; any other token is ignored.
async fn provider_sign_in(
    state: &AppState,
    provider: Provider,
    caller: Option<Identity>,
    code: &str,
) -> ApiResult<Response> {
    let caller = caller.map(|identity| identity.user_id);
    let user = oauth_service::sign_in(
        &state.pool,
        &state.http,
        &state.config,
        provider,
        code,
        caller,
    )
    .await?;
    session_response(state, user, StatusCode::OK)
}
