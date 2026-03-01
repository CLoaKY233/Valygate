use axum::{Json, response::IntoResponse};
use serde_json::json;

pub async fn root_handler() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "valygate"
    }))
}
