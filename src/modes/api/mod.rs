mod server;
mod wiring;

use crate::config::AppConfig;

pub async fn run(config: AppConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (addr, app) = wiring::build_router(config).await?;
    server::listen_and_serve(&addr, app).await
}
