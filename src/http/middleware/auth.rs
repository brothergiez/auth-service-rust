use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::header;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use bson::oid::ObjectId;

use crate::error::AppError;
use crate::jwt;
use crate::state::AppState;

/// Inserted by [`require_jwt`] for downstream handlers.
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: ObjectId,
}

pub async fn require_jwt(
    State(app): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(raw) = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    else {
        return AppError::Unauthorized("missing Authorization header".into()).into_response();
    };

    let token = if let Some(rest) = raw.strip_prefix("Bearer ") {
        rest
    } else if let Some(rest) = raw.strip_prefix("bearer ") {
        rest
    } else {
        return AppError::Unauthorized("expected `Authorization: Bearer <token>`".into()).into_response();
    };

    let claims = match jwt::decode_access_token(token, &app.jwt_secret) {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };

    let user_id = match ObjectId::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return AppError::Unauthorized("invalid subject in token".into()).into_response(),
    };

    request.extensions_mut().insert(AuthUser { user_id });
    next.run(request).await
}
