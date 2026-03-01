use anyhow;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Upstream provider timed out")]
    ProviderTimeout,

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::ProviderTimeout => (StatusCode::GATEWAY_TIMEOUT, "Provider timeout".into()),
            AppError::Internal(err) => {
                tracing::error!("Internal server error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into())
            }
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "status": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}
