use std::sync::Arc;

use async_trait::async_trait;
use tokio_cron_scheduler::{Job, JobScheduler};

/// One unit of work executed on a cron expression (implement for DB cleanup, outbox flush, etc.).
#[async_trait]
pub trait CronTask: Send + Sync {
    async fn tick(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Default task: logs each firing.
pub struct LoggingCronTask;

#[async_trait]
impl CronTask for LoggingCronTask {
    async fn tick(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!(mode = "cron", "scheduled job tick");
        Ok(())
    }
}

/// Attach a task to the scheduler (expression = six-field `sec min hour day month weekday`).
pub async fn register<T: CronTask + 'static>(
    sched: &mut JobScheduler,
    schedule: &str,
    task: Arc<T>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let schedule_owned = schedule.to_string();
    sched
        .add(
            Job::new_async(schedule_owned.as_str(), move |_uuid, _lock| {
                let task = task.clone();
                Box::pin(async move {
                    if let Err(e) = task.tick().await {
                        tracing::warn!(mode = "cron", error = %e, "cron task error");
                    }
                })
            })?,
        )
        .await?;
    Ok(())
}
