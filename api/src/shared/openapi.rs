//! Pieces of the OpenAPI document that are not owned by any single domain.

use serde::Serialize;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, ToSchema};

/// Every error response has this shape, whatever the status code.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Human-readable reason, already translated.
    #[schema(example = "Ce pseudo est déjà pris")]
    pub detail: String,
}

/// Declares the bearer scheme once, so each protected route only references it by name.
pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // `components` may still be empty when this runs.
        let components = openapi.components.get_or_insert_with(Default::default);

        components.add_security_scheme(
            "bearer",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("Access token returned by /auth/dev-login"))
                    .build(),
            ),
        );
    }
}
