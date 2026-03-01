use anyhow::Result;
use axum::routing::get;
use tower_http::trace::TraceLayer;
use tracing::info;
use valygate::{rts::root::root_handler, sys::init::initialize};

#[tokio::main]
async fn main() -> Result<()> {
    let env_loaded = dotenvy::dotenv().is_ok();
    telemetry::init_tracing();
    if env_loaded {
        info!(".env file loaded successfully");
    }

    let (state, listener) = initialize().await?;

    info!(
        address = %listener.local_addr()?,
        version = env!("CARGO_PKG_VERSION"),
        "Server is ready"
    );

    let app = axum::Router::new()
        .route("/", get(root_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!("Failed to handle Ctrl+C: {}", e);
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sigterm) => {
                sigterm.recv().await;
            }
            Err(e) => {
                tracing::error!("Failed to install SIGTERM handler: {}", e);
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::warn!("Shutdown signal received, starting graceful shutdown of Valygate...");
}
