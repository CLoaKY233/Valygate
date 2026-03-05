use axum::{Json, response::IntoResponse};
use serde_json::json;

#[must_use]
#[allow(clippy::unused_async)]
pub async fn root_handler() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "valygate"
    }))
}
