//! Router assembly: mount the controllers, then wrap the whole thing in the middleware stack.
//!
//! Kept apart from `main.rs` so tests call the very same router in-process.

use anyhow::{Context, Result};
use axum::{Json, Router};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use http::{HeaderValue, Method};
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use serde::Serialize;
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

use crate::controllers;
use crate::shared::openapi::SecurityAddon;
use crate::state::AppState;

const REQUEST_ID_HEADER: &str = "x-request-id";

/// What no route carries: title, tag descriptions, the bearer scheme. Routes add themselves.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "scout-beam",
        description = "Scout Beam API: fire beams, deduce the hidden layout",
        version = "0.1.0",
    ),
    tags(
        (name = "Service", description = "Process-level endpoints"),
        (name = "Auth", description = "Sessions and tokens"),
        (name = "Games", description = "Games against the bot, the shots fired at them and the propositions"),
    ),
    modifiers(&SecurityAddon),
)]
struct ApiDoc;

pub fn build_app(state: AppState) -> Result<Router> {
    let cors = cors_layer(&state.config.cors_allowed_origin)?;
    let is_dev = state.config.environment.is_dev();

    let (router, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(get_health))
        .merge(controllers::router(is_dev))
        .split_for_parts();
    let router = router.with_state(state).merge(docs_router(openapi));

    // Outermost first: an id is attached, the request is traced, CORS is answered, slow work is cut off.
    Ok(router
        .layer(TimeoutLayer::with_status_code(
            http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::new(
            http::HeaderName::from_static(REQUEST_ID_HEADER),
            MakeRequestUuid,
        )))
}

/// Swagger UI plus the document it reads, `no-store` so a new route never looks missing.
fn docs_router(openapi: utoipa::openapi::OpenApi) -> Router {
    let swagger: Router = SwaggerUi::new("/docs")
        .url("/api-docs/openapi.json", openapi)
        .into();

    swagger.layer(SetResponseHeaderLayer::overriding(
        http::header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    ))
}

/// Answer of `GET /health`.
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    #[schema(example = "ok")]
    pub status: &'static str,
}

/// Liveness only — it must not depend on the database.
#[utoipa::path(
    get,
    path = "/health",
    tag = "Service",
    responses((status = 200, description = "The process is up", body = HealthResponse)),
)]
async fn get_health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

fn cors_layer(origin: &str) -> Result<CorsLayer> {
    let origin: HeaderValue = origin
        .parse()
        .with_context(|| format!("CORS_ALLOWED_ORIGIN is not a valid origin: {origin}"))?;

    Ok(CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_credentials(true))
}
