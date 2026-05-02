use std::str::FromStr;

use crate::error::AppError;

use super::AppConfig;

/// Process role: HTTP API, Kafka consumer, or scheduled jobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Api,
    Worker,
    Cron,
}

impl FromStr for AppMode {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "api" => Ok(Self::Api),
            "worker" => Ok(Self::Worker),
            "cron" => Ok(Self::Cron),
            _ => Err(AppError::Config(format!(
                "APP_MODE must be one of: api, worker, cron (got '{s}')"
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkerSettings {
    pub kafka_brokers: String,
    pub kafka_topic: String,
    pub kafka_group_id: String,
}

#[derive(Debug, Clone)]
pub struct CronSettings {
    /// Six-field cron: `sec min hour day month weekday` (see `tokio-cron-scheduler`).
    pub schedule: String,
}

/// Fully validated configuration for the selected mode.
#[derive(Debug, Clone)]
pub enum LoadedApp {
    Api(AppConfig),
    Worker(WorkerSettings),
    Cron(CronSettings),
}

impl LoadedApp {
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();
        let mode_raw = std::env::var("APP_MODE").unwrap_or_else(|_| "api".into());
        let mode: AppMode = mode_raw.parse()?;
        Ok(match mode {
            AppMode::Api => Self::Api(super::env::load_api()?),
            AppMode::Worker => Self::Worker(super::env::load_worker()?),
            AppMode::Cron => Self::Cron(super::env::load_cron()?),
        })
    }
}
