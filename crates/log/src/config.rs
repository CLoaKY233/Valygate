use super::models::{LogConfig, LogFormat};
use std::env;

impl LogFormat {
    #[must_use]
    pub fn from_env() -> Self {
        let raw = env::var("LOG_FORMAT").unwrap_or_default();

        match raw.to_lowercase().as_str() {
            "json" => Self::Json,
            "compact" => Self::Compact,
            "pretty" => Self::Pretty,
            _ => {
                if cfg!(debug_assertions) {
                    Self::Compact
                } else {
                    Self::Json
                }
            }
        }
    }
}

impl LogConfig {
    #[must_use]
    pub fn from_env() -> Self {
        let format = LogFormat::from_env();

        let filter = env::var("RUST_LOG").unwrap_or_else(|_| {
            if cfg!(debug_assertions) {
                "valygate=debug,tower_http=debug,info".to_string()
            } else {
                "valygate=info,tower_http=info,warn".to_string()
            }
        });

        Self { format, filter }
    }
}
