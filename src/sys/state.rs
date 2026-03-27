use crate::sys::{client::HttpClient, config::AppConfig};
use std::sync::Arc;
use valymux_surrealdb::Database;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub http_client: Arc<dyn HttpClient>,
    pub reqwest_client: reqwest::Client,
    pub database: Database,
}
