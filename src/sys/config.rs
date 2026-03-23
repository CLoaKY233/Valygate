use std::fmt;

use serde::Deserialize;
use valygate_surrealdb::DatabaseConfig;

fn default_host() -> String {
    "0.0.0.0".to_string()
}
fn default_port() -> u16 {
    3000
}
fn default_pool_idle_timeout_secs() -> u64 {
    90
}
fn default_pool_max_idle_per_host() -> usize {
    32
}
fn default_connect_timeout_secs() -> u64 {
    10
}
fn default_timeout_secs() -> u64 {
    300
}

#[derive(Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_host")]
    pub server_host: String,

    #[serde(default = "default_port")]
    pub server_port: u16,

    #[serde(default = "default_pool_idle_timeout_secs")]
    pub http_pool_idle_timeout_secs: u64,

    #[serde(default = "default_pool_max_idle_per_host")]
    pub http_pool_max_idle_per_host: usize,

    #[serde(default = "default_connect_timeout_secs")]
    pub http_connect_timeout_secs: u64,

    #[serde(default = "default_timeout_secs")]
    pub http_timeout_secs: u64,

    pub surreal_url: String,

    pub surreal_namespace: String,

    pub surreal_database: String,

    pub surreal_username: String,

    pub surreal_password: String,

    pub surreal_encryption_key: String,
}

impl fmt::Debug for AppConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AppConfig")
            .field("server_host", &self.server_host)
            .field("server_port", &self.server_port)
            .field(
                "http_pool_idle_timeout_secs",
                &self.http_pool_idle_timeout_secs,
            )
            .field(
                "http_pool_max_idle_per_host",
                &self.http_pool_max_idle_per_host,
            )
            .field("http_connect_timeout_secs", &self.http_connect_timeout_secs)
            .field("http_timeout_secs", &self.http_timeout_secs)
            .field("surreal_url", &self.surreal_url)
            .field("surreal_namespace", &self.surreal_namespace)
            .field("surreal_database", &self.surreal_database)
            .field("surreal_username", &self.surreal_username)
            .field("surreal_password", &"<REDACTED>")
            .field("surreal_encryption_key", &"<REDACTED>")
            .finish()
    }
}

impl AppConfig {
    /// # Errors
    /// Returns an error if environment variable deserialization fails.
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env::<AppConfig>()
    }

    #[must_use]
    pub fn address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }

    #[must_use]
    pub fn database_config(&self) -> DatabaseConfig {
        DatabaseConfig {
            surreal_url: self.surreal_url.clone(),
            surreal_namespace: self.surreal_namespace.clone(),
            surreal_database: self.surreal_database.clone(),
            surreal_username: self.surreal_username.clone(),
            surreal_password: self.surreal_password.clone(),
            surreal_encryption_key: self.surreal_encryption_key.clone(),
        }
    }
}
