use crate::sys::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
}
