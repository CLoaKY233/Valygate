use axum::response::IntoResponse;

pub async fn root_handler() -> impl IntoResponse {
    "Welcome to Valygate!"
}
