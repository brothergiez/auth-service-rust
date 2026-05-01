use crate::error::AppError;
use std::time::Duration;

#[derive(Clone)]
pub struct AppConfig {
    pub mongodb_uri: String,
    pub database_name: String,
    pub jwt_secret: String,
    pub jwt_expiration: Duration,
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();

        let mongodb_uri = std::env::var("MONGODB_URI")
            .map_err(|_| AppError::Config("MONGODB_URI is required".into()))?;
        let database_name = std::env::var("DATABASE_NAME").unwrap_or_else(|_| "auth_service".into());
        let jwt_secret =
            std::env::var("JWT_SECRET").map_err(|_| AppError::Config("JWT_SECRET is required".into()))?;
        if jwt_secret.len() < 32 {
            return Err(AppError::Config(
                "JWT_SECRET must be at least 32 characters for HS256".into(),
            ));
        }

        let jwt_expiration_secs: u64 = std::env::var("JWT_EXPIRATION_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3600);

        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
        let port = std::env::var("PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3000);

        Ok(Self {
            mongodb_uri,
            database_name,
            jwt_secret,
            jwt_expiration: Duration::from_secs(jwt_expiration_secs),
            host,
            port,
        })
    }
}
