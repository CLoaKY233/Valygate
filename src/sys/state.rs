use crate::sys::{client::HttpClient, config::AppConfig};
use std::sync::Arc;
use valygate_surrealdb::Database;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub http_client: Arc<dyn HttpClient>,
    pub database: Database,
}
