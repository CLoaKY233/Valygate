use anyhow::{Context, Result, bail};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::info;
use valymux_surrealdb::Database;

use super::{config::AppConfig, state::AppState};

/// # Errors
/// Returns an error if:
/// - Environment variable deserialization fails
/// - HTTP timeout values are invalid
/// - TCP listener fails to bind
/// - HTTP client fails to build
/// - Database configuration is invalid
pub async fn initialize() -> Result<(Arc<AppState>, TcpListener)> {
    let config = AppConfig::from_env()?;

    if config.http_connect_timeout_secs == 0 || config.http_timeout_secs == 0 {
        bail!("HTTP timeout values must be > 0");
    }
    if config.http_timeout_secs < config.http_connect_timeout_secs {
        bail!("http_timeout_secs must be >= http_connect_timeout_secs");
    }

    info!(
        host = %config.server_host,
        port = config.server_port,
        "Server configuration loaded"
    );

    let address = config.address();
    let listener = TcpListener::bind(&address)
        .await
        .with_context(|| format!("Failed to bind TCP listener to {address}"))?;

    info!(address = %address, "TCP listener bound");

    let http_client = Client::builder()
        .pool_idle_timeout(Duration::from_secs(config.http_pool_idle_timeout_secs))
        .pool_max_idle_per_host(config.http_pool_max_idle_per_host)
        .connect_timeout(Duration::from_secs(config.http_connect_timeout_secs))
        .timeout(Duration::from_secs(config.http_timeout_secs))
        .build()
        .context("Failed to build HTTP client")?;

    // Create Database instance - NO connection or bootstrap needed.
    // Schema must be applied manually via Surrealist/CLI before server startup.
    let database_config = config.database_config();
    let database =
        Database::new(database_config.clone()).context("Failed to create Database instance")?;
    if database_config.has_service_credentials() {
        database
            .initialize_runtime()
            .await
            .context("Failed to authenticate backend service database client")?;
    } else {
        bail!("VALYMUX_DB_SERVICE_KEY is not set; refusing to start — proxy secret fetching requires the backend-service bearer grant");
    }
    info!("Database configuration validated");

    let state = Arc::new(AppState {
        config,
        reqwest_client: http_client.clone(),
        http_client: std::sync::Arc::new(http_client),
        database,
    });

    Ok((state, listener))
}
