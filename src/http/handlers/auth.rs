use axum::extract::State;
use axum::Extension;
use axum::Json;
use std::sync::Arc;

use crate::error::AppError;
use crate::http::middleware::AuthUser;
use crate::http::schemas::{AuthResponse, LoginRequest, RegisterRequest, UserPublic};
use crate::state::AppState;

/// Register a new user and return a JWT.
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Registered", body = AuthResponse),
        (status = 400, description = "Validation error", body = crate::error::ErrorBody),
        (status = 409, description = "Email already exists", body = crate::error::ErrorBody),
    ),
    tag = "auth"
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let res = state.auth.register(body).await?;
    Ok(Json(res))
}

/// Login with email and password; returns a JWT.
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "OK", body = AuthResponse),
        (status = 400, description = "Validation error", body = crate::error::ErrorBody),
        (status = 401, description = "Invalid credentials", body = crate::error::ErrorBody),
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let res = state.auth.login(body).await?;
    Ok(Json(res))
}

/// Current user profile (requires `Authorization: Bearer <access_token>`).
#[utoipa::path(
    get,
    path = "/api/v1/auth/getme",
    responses(
        (status = 200, description = "OK", body = UserPublic),
        (status = 401, description = "Missing/invalid token or user gone", body = crate::error::ErrorBody),
    ),
    security(("bearer_auth" = [])),
    tag = "auth"
)]
pub async fn get_me(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<UserPublic>, AppError> {
    let me = state.auth.get_me(auth.user_id).await?;
    Ok(Json(me))
}
