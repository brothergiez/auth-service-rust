mod env;
mod mode;

use std::time::Duration;

pub use mode::{AppMode, CronSettings, LoadedApp, WorkerSettings};

use crate::error::AppError;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub mongodb_uri: String,
    pub database_name: String,
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
