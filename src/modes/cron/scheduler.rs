use tokio_cron_scheduler::JobScheduler;

/// Start the scheduler, wait for Ctrl+C, then shut down cleanly.
pub async fn run_until_ctrl_c(
    mut sched: JobScheduler,
    schedule_label: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sched.start().await?;
    tracing::info!(
        mode = "cron",
        schedule = %schedule_label,
        "scheduler running; Ctrl+C to exit"
    );

    tokio::signal::ctrl_c().await?;
    sched.shutdown().await?;
    Ok(())
}
