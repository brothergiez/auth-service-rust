mod scheduler;
mod task;

use std::sync::Arc;

use tokio_cron_scheduler::JobScheduler;

pub use task::{CronTask, LoggingCronTask};

use crate::config::CronSettings;

/// Run cron mode with the default [`LoggingCronTask`].
pub async fn run(settings: CronSettings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run_with_task(settings, Arc::new(LoggingCronTask)).await
}

/// Run cron mode with a custom [`CronTask`].
pub async fn run_with_task<T: CronTask + 'static>(
    settings: CronSettings,
    task: Arc<T>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let schedule = settings.schedule.clone();
    let mut sched = JobScheduler::new().await?;
    task::register(&mut sched, &schedule, task).await?;
    scheduler::run_until_ctrl_c(sched, &schedule).await
}
