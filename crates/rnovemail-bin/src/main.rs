use std::{net::SocketAddr, path::Path, sync::Arc};

use anyhow::Context;
use rnovemail_api::{AppState, build_router};
use rnovemail_config::AppConfig;
use rnovemail_observability::init_logging;
use rnovemail_store_rnmdb::{RnovStore, RnovStoreKey};
use secrecy::ExposeSecret;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::from_env().context("load configuration")?;
    init_logging(&config.observability);
    let state = build_state(&config).await.context("initialize state")?;
    serve(config.http.bind, state).await
}

async fn serve(bind: SocketAddr, state: AppState) -> anyhow::Result<()> {
    let app = build_router(state);
    let listener = TcpListener::bind(bind).await?;
    tracing::info!(%bind, "rnovemail listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn build_state(config: &AppConfig) -> anyhow::Result<AppState> {
    let key = read_store_key(&config.security.master_key_file)?;
    let store = Arc::new(RnovStore::open(&config.storage.data_dir, key)?);
    let token = config
        .security
        .bootstrap_admin_token
        .as_ref()
        .map(|token| token.expose_secret().to_string());
    AppState::with_persistent_store(token, store)
        .await
        .map(|state| state.with_public_base_url(config.http.public_base_url.as_str()))
        .map_err(Into::into)
}

fn read_store_key(path: &Path) -> anyhow::Result<RnovStoreKey> {
    let material = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    RnovStoreKey::derive_from_master_key(&material).context("derive rnmdb page key")
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
