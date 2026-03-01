use crate::models::{LogConfig, LogFormat};
use tracing_subscriber::{EnvFilter, fmt};

pub fn init_tracing() {
    let config = LogConfig::from_env();
    let filter = EnvFilter::try_new(&config.filter).unwrap_or_else(|_| EnvFilter::new("info"));

    match config.format {
        LogFormat::Pretty => {
            fmt()
                .pretty()
                .with_env_filter(filter)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_level(true)
                .init();
        }
        LogFormat::Json => {
            fmt()
                .json()
                .with_env_filter(filter)
                .with_target(true)
                .with_file(false)
                .with_line_number(false)
                .with_thread_ids(false)
                .with_level(true)
                .with_current_span(true)
                .init();
        }
        LogFormat::Compact => {
            fmt()
                .compact()
                .with_env_filter(filter)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(false)
                .with_level(true)
                .init();
        }
    }

    tracing::debug!(
        format = ?config.format,
        filter = %config.filter,
        "Tracing initialized"
    );
}
