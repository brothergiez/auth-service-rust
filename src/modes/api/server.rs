use axum::Router;
use tokio::net::TcpListener;

pub async fn listen_and_serve(
    addr: &str,
    app: Router,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(mode = "api", %addr, "listening");
    tracing::info!(mode = "api", "swagger ui: http://{}/swagger-ui", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
