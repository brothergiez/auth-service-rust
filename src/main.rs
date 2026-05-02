use auth_service::config::LoadedApp;
use auth_service::modes::{run_api, run_cron, run_worker};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    match LoadedApp::from_env()? {
        LoadedApp::Api(config) => run_api(config).await,
        LoadedApp::Worker(settings) => run_worker(settings).await,
        LoadedApp::Cron(settings) => run_cron(settings).await,
    }
}
