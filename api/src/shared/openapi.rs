//! Pieces of the OpenAPI document that are not owned by any single domain.

use serde::Serialize;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, ToSchema};

/// Every error response has this shape, whatever the status code.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Stable reason the front translates; a failed validation names its first rule.
    #[schema(example = "missing_colour")]
    pub code: String,
    /// What fills the code's text, such as the shapes or colour a layout rule names.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = json!({ "colour": "yellow" }))]
    pub params: Option<serde_json::Value>,
    /// Human-readable reason in English, or a `[{loc, msg, type}]` list for a failed validation.
    #[schema(example = "the layout has no Yellow piece")]
    pub detail: String,
    /// Only on a refusal that lifts by itself: milliseconds before trying again.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 5000)]
    pub retry_after_ms: Option<u64>,
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
                    .description(Some(
                        "The value of the scout_beam_access cookie a sign-in sets; browsers send the cookie itself",
                    ))
                    .build(),
            ),
        );
    }
}
