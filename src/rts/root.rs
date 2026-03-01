use axum::{response::IntoResponse, Json};
use serde_json::json;

pub async fn root_handler() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "valygate"
    }))
}
