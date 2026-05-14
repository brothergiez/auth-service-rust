use std::sync::Arc;

use axum::Router;
use mongodb::Client;
use redis::aio::ConnectionManager;

use crate::config::{AppConfig, DatabaseSettings};
use crate::http::routes;
use crate::repository::{MongoUserRepository, MysqlUserRepository, UserRepository};
use crate::service::{AuthService, AuthServiceImpl};
use crate::state::AppState;

/// Build the HTTP app and the `host:port` listen address from API config.
pub async fn build_router(
    config: AppConfig,
) -> Result<(String, Router), Box<dyn std::error::Error + Send + Sync>> {
    let users: Arc<dyn UserRepository> = match &config.database {
        DatabaseSettings::Mongo { uri, database_name } => {
            let mongo = Client::with_uri_str(uri).await?;
            Arc::new(MongoUserRepository::new(&mongo, database_name).await?)
        }
        DatabaseSettings::Mysql { url } => Arc::new(MysqlUserRepository::connect_and_migrate(url).await?),
    };

    let auth_impl = Arc::new(AuthServiceImpl::new(users, config.clone()));
    let auth: Arc<dyn AuthService> = auth_impl;

    let redis = if let Some(url) = config.redis_url.as_deref() {
        let client = redis::Client::open(url)?;
        let manager = ConnectionManager::new(client).await?;
        tracing::info!(mode = "api", "redis: connected");
        Some(manager)
    } else {
        tracing::info!(mode = "api", "redis: disabled (REDIS_URL unset)");
        None
    };

    let state = Arc::new(AppState::new(
        auth,
        config.jwt_secret.clone(),
        redis,
    ));
    let addr = format!("{}:{}", config.host, config.port);
    let app = routes::router(state);
    Ok((addr, app))
}
