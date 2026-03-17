use crate::sys::config::AppConfig;
use reqwest::Client;

// TODO: Wrap `http_client` behind an `HttpClient` trait to decouple handlers from reqwest,
// enabling mock injection in tests and backend swappability. Revisit when the first
// upstream-calling handler is added.
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub http_client: Client,
}
