use axum::{extract::FromRequestParts, http::request::Parts};
use std::sync::Arc;
use valygate_core::error::AppError;
use valygate_surrealdb::User;

use crate::sys::state::AppState;

pub struct RequireAuth {
    pub user: User,
    pub token: String,
}

impl FromRequestParts<Arc<AppState>> for RequireAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "));

        let Some(token) = auth_header else {
            return Err(AppError::Unauthorized(
                "Missing or invalid Authorization header".to_string(),
            ));
        };

        let user = state
            .database
            .authenticate_user(token)
            .await
            .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;

        Ok(RequireAuth {
            user,
            token: token.to_string(),
        })
    }
}
