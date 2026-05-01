use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::error::ErrorBody;
use crate::http::handlers;
use crate::http::schemas::{AuthResponse, HealthResponse, LoginRequest, RegisterRequest};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    paths(
        handlers::health::health,
        handlers::auth::register,
        handlers::auth::login,
        handlers::auth::get_me,
    ),
    components(schemas(
        RegisterRequest,
        LoginRequest,
        AuthResponse,
        crate::http::schemas::UserPublic,
        HealthResponse,
        ErrorBody,
    )),
    tags(
        (name = "auth", description = "Authentication"),
        (name = "health", description = "Service health"),
    ),
)]
pub struct ApiDoc;
