use crate::sys::config::AppConfig;
use reqwest::Client;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub http_client: Client,
}
