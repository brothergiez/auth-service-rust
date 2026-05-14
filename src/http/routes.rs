use std::sync::Arc;

use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::http::handlers;
use crate::http::middleware::{log_request, require_jwt};
use crate::http::ApiDoc;
use crate::state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    let jwt_state = state.clone();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/health", get(handlers::health))
        .route("/api/v1/auth/register", post(handlers::register))
        .route("/api/v1/auth/login", post(handlers::login))
        .route(
            "/api/v1/auth/getme",
            get(handlers::get_me).route_layer(middleware::from_fn_with_state(
                jwt_state,
                require_jwt,
            )),
        )
        .layer(cors);

    let openapi = ApiDoc::openapi();
    Router::<Arc<AppState>>::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .merge(api)
        .layer(middleware::from_fn(log_request))
        .with_state(state)
}
