use axum::Json;

use crate::http::schemas::HealthResponse;

/// Liveness / readiness style health check.
#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "OK", body = HealthResponse))
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
    })
}
