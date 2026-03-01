use serde::Deserialize;

fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 3000 }

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_host")]
    pub server_host: String,
    
    #[serde(default = "default_port")]
    pub server_port: u16,
}

impl AppConfig {
    pub fn from_env() -> Self {
        envy::from_env::<AppConfig>().unwrap_or_else(|_| AppConfig {
            server_host: default_host(),
            server_port: default_port(),
        })
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
