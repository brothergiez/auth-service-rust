use std::sync::Arc;

use auth_service::config::AppConfig;
use auth_service::http::routes;
use auth_service::repository::MongoUserRepository;
use auth_service::service::{AuthService, AuthServiceImpl};
use auth_service::state::AppState;
use mongodb::Client;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env()?;
    let mongo = Client::with_uri_str(&config.mongodb_uri).await?;
    let users = Arc::new(MongoUserRepository::new(&mongo, &config.database_name).await?);
    let auth_impl = Arc::new(AuthServiceImpl::new(users, config.clone()));
    let auth: Arc<dyn AuthService> = auth_impl;

    let state = Arc::new(AppState::new(auth, config.jwt_secret.clone()));
    let app = routes::router(state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("listening on http://{}", addr);
    tracing::info!("swagger ui: http://{}/swagger-ui", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
