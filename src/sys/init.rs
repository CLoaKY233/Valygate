use anyhow::{Context, Result};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::info;

use super::{config::AppConfig, state::AppState};

pub async fn initialize() -> Result<(Arc<AppState>, TcpListener)> {
    let config = AppConfig::from_env();

    info!(
        host = %config.server_host,
        port = config.server_port,
        "Server configuration loaded"
    );

    let address = config.address();
    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("Failed to bind TCP listener to {}", address))?;

    let http_client = Client::builder()
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(32)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(300))
        .build()
        .context("Failed to build HTTP client")?;

    let state = Arc::new(AppState {
        config,
        http_client,
    });

    Ok((state, listener))
}
