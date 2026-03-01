use serde::Deserialize;

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

#[derive(Clone, Debug, Deserialize)]
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
}

impl AppConfig {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env::<AppConfig>()
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
