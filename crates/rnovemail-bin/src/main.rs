use std::net::SocketAddr;

use anyhow::Context;
use rnovemail_api::{AppState, build_router};
use rnovemail_config::AppConfig;
use rnovemail_observability::init_logging;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::from_env().context("load configuration")?;
    init_logging(&config.observability);
    serve(config.http.bind).await
}

async fn serve(bind: SocketAddr) -> anyhow::Result<()> {
    let app = build_router(AppState::empty());
    let listener = TcpListener::bind(bind).await?;
    tracing::info!(%bind, "rnovemail listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
