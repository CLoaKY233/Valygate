use anyhow::{Context, Result};
use std::sync::Arc;
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

    info!("Listening on http://{}", address);

    let state = Arc::new(AppState { config });

    Ok((state, listener))
}
