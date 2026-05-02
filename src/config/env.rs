use std::time::Duration;

use crate::error::AppError;

use super::mode::{CronSettings, WorkerSettings};
use super::AppConfig;

pub fn load_api() -> Result<AppConfig, AppError> {
    let mongodb_uri = std::env::var("MONGODB_URI")
        .map_err(|_| AppError::Config("MONGODB_URI is required in api mode".into()))?;
    let database_name = std::env::var("DATABASE_NAME").unwrap_or_else(|_| "auth_service".into());
    let jwt_secret =
        std::env::var("JWT_SECRET").map_err(|_| AppError::Config("JWT_SECRET is required in api mode".into()))?;
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

    Ok(AppConfig {
        mongodb_uri,
        database_name,
        jwt_secret,
        jwt_expiration: Duration::from_secs(jwt_expiration_secs),
        host,
        port,
    })
}

pub fn load_worker() -> Result<WorkerSettings, AppError> {
    Ok(WorkerSettings {
        kafka_brokers: std::env::var("KAFKA_BROKERS").map_err(|_| {
            AppError::Config("KAFKA_BROKERS is required in worker mode (e.g. localhost:9092)".into())
        })?,
        kafka_topic: std::env::var("KAFKA_TOPIC")
            .map_err(|_| AppError::Config("KAFKA_TOPIC is required in worker mode".into()))?,
        kafka_group_id: std::env::var("KAFKA_GROUP_ID")
            .map_err(|_| AppError::Config("KAFKA_GROUP_ID is required in worker mode".into()))?,
    })
}

pub fn load_cron() -> Result<CronSettings, AppError> {
    let schedule = std::env::var("CRON_SCHEDULE").unwrap_or_else(|_| "0 * * * * *".into());
    Ok(CronSettings { schedule })
}
