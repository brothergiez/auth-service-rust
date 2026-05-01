mod env;

use std::time::Duration;

use crate::error::AppError;

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
        let e = env::load()?;
        Ok(Self {
            mongodb_uri: e.mongodb_uri,
            database_name: e.database_name,
            jwt_secret: e.jwt_secret,
            jwt_expiration: Duration::from_secs(e.jwt_expiration_secs),
            host: e.host,
            port: e.port,
        })
    }
}
