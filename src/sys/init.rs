use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

use super::{config::AppConfig, state::AppState};

pub async fn initialize() -> Result<(Arc<AppState>, TcpListener)> {
    let env_loaded = dotenvy::dotenv().is_ok();
    if env_loaded {
        info!(".env file loaded");
    }

    let config = AppConfig::from_env();
    info!(
        host = %config.host,
        port = config.port,
        "Server configuration loaded"
    );
    let address = config.address();
    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("Failed to bind TCP listener to {}", address))?;

    info!("Listening on http://{}", address);

    let state = Arc::new(AppState {
        config: Arc::new(config),
    });

    Ok((state, listener))
}
