use anyhow::Result;
use axum::routing::get;
use tracing::info;
use valygate::{rts::root::root_handler, sys::init::initialize};

#[tokio::main]
async fn main() -> Result<()> {
    log::init_tracing();

    let (state, listener) = initialize().await?;

    info!(
        address = %listener.local_addr()?,
        version = env!("CARGO_PKG_VERSION"),
        "Server is ready"
    );

    let app = axum::Router::new()
        .route("/", get(root_handler))
        .with_state(state);

    axum::serve(listener, app).await?;

    Ok(())
}
