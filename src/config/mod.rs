mod env;
mod mode;

use std::time::Duration;

pub use mode::{AppMode, CronSettings, LoadedApp, WorkerSettings};

use crate::error::AppError;

/// Which persistence backend the API uses (selected at deploy time via `DATABASE_DRIVER`).
#[derive(Clone, Debug)]
pub enum DatabaseSettings {
    Mongo {
        uri: String,
        database_name: String,
    },
    Mysql {
        url: String,
    },
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database: DatabaseSettings,
    /// When `Some`, the API opens a Redis connection manager on startup (`REDIS_URL` in `.env`).
    pub redis_url: Option<String>,
    pub jwt_secret: String,
    pub jwt_expiration: Duration,
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    /// Load **API-only** config (use [`LoadedApp::from_env`] for multi-mode).
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();
        env::load_api()
    }
}
