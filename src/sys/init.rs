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
        .pool_idle_timeout(Duration::from_secs(config.http_pool_idle_timeout_secs))
        .pool_max_idle_per_host(config.http_pool_max_idle_per_host)
        .connect_timeout(Duration::from_secs(config.http_connect_timeout_secs))
        .timeout(Duration::from_secs(config.http_timeout_secs))
        .build()
        .context("Failed to build HTTP client")?;

    let state = Arc::new(AppState {
        config,
        http_client,
    });

    Ok((state, listener))
}
