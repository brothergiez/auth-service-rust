use std::sync::Arc;

use axum::Router;
use mongodb::Client;

use crate::config::AppConfig;
use crate::http::routes;
use crate::repository::MongoUserRepository;
use crate::service::{AuthService, AuthServiceImpl};
use crate::state::AppState;

/// Build the HTTP app and the `host:port` listen address from API config.
pub async fn build_router(
    config: AppConfig,
) -> Result<(String, Router), Box<dyn std::error::Error + Send + Sync>> {
    let mongo = Client::with_uri_str(&config.mongodb_uri).await?;
    let users = Arc::new(MongoUserRepository::new(&mongo, &config.database_name).await?);
    let auth_impl = Arc::new(AuthServiceImpl::new(users, config.clone()));
    let auth: Arc<dyn AuthService> = auth_impl;

    let state = Arc::new(AppState::new(auth, config.jwt_secret.clone()));
    let addr = format!("{}:{}", config.host, config.port);
    let app = routes::router(state);
    Ok((addr, app))
}
