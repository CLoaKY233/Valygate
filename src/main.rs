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

    axum::serve(listener, app).await?;

    Ok(())
}
