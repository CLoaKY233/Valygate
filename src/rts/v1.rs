use std::{sync::Arc, time::Instant};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb_types::ToSql;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use valygate_core::error::AppError;
use valygate_surrealdb::{
    CreateProviderCredentialInput, CreateVirtualApiKeyInput, ModelCatalogEntry, ProviderCredential,
    ProviderKind, SigninInput, SignupInput, UpdateProfileInput, UpdateProviderCredentialInput,
    UpdateVirtualApiKeyInput, User, VirtualApiKey,
};

use crate::{svc::proxy, sys::state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/signup", post(signup))
        .route("/auth/signin", post(signin))
        .route("/me", get(me).patch(update_me))
        .route("/providers", get(list_providers).post(create_provider))
        .route(
            "/providers/{provider_id}",
            get(get_provider)
                .patch(update_provider)
                .delete(delete_provider),
        )
        .route(
            "/virtual-keys",
            get(list_virtual_keys).post(create_virtual_key),
        )
        .route(
            "/virtual-keys/{key_id}",
            get(get_virtual_key)
                .patch(update_virtual_key)
                .delete(delete_virtual_key),
        )
        .route("/models", get(list_models))
        .route("/models/{alias}", get(get_model))
        .route("/v1/chat/completions", post(chat_completions))
}

#[derive(Serialize)]
struct AuthResponse {
    user: UserResponse,
    token: String,
}

#[derive(Serialize)]
struct UserResponse {
    id: String,
    name: String,
    email: String,
    enabled: bool,
}

#[derive(Serialize)]
struct ProviderResponse {
    id: String,
    provider: String,
    label: String,
    tags: Vec<String>,
    enabled: bool,
    last_used_at: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct VirtualKeyResponse {
    id: String,
    name: String,
    key_prefix: String,
    allowed_models: Vec<String>,
    tags: Vec<String>,
    enabled: bool,
    expires_at: Option<String>,
    last_used_at: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct CreateVirtualKeyResponse {
    key: VirtualKeyResponse,
    raw_key: String,
}

#[derive(Serialize)]
struct ModelResponse {
    id: String,
    alias: String,
    display_name: String,
    provider: String,
    upstream_model: String,
    description: String,
    tags: Vec<String>,
    enabled: bool,
    context_window_tokens: i64,
    max_output_tokens: i64,
    supports_streaming: bool,
    supports_thinking: bool,
    thinking_required: bool,
    supports_temperature: bool,
    temperature_fixed_to: Option<f64>,
    temperature_min: Option<f64>,
    temperature_max: Option<f64>,
    supports_top_p: bool,
    supports_system_messages: bool,
    supports_tools: bool,
    supports_vision: bool,
    supports_json_mode: bool,
    supports_parallel_tool_calls: bool,
}

#[derive(Deserialize)]
struct CreateProviderRequest {
    provider: ProviderKind,
    label: String,
    api_key: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Deserialize)]
struct UpdateProviderRequest {
    label: String,
    api_key: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    enabled: bool,
}

#[derive(Deserialize)]
struct CreateVirtualKeyRequest {
    name: String,
    #[serde(default)]
    allowed_models: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize)]
struct UpdateVirtualKeyRequest {
    name: String,
    #[serde(default)]
    allowed_models: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    enabled: bool,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tracing::instrument(skip(state, input))]
async fn signup(
    State(state): State<Arc<AppState>>,
    Json(input): Json<SignupInput>,
) -> Result<Json<AuthResponse>, AppError> {
    debug!("signup request received");
    let session = state.database.signup_user(input).await.map_err(|error| {
        error!(error = %error, "signup failed");
        internal_error(error)
    })?;
    Ok(Json(AuthResponse {
        user: map_user(&session.user),
        token: session.token,
    }))
}

#[tracing::instrument(skip(state, input))]
async fn signin(
    State(state): State<Arc<AppState>>,
    Json(input): Json<SigninInput>,
) -> Result<Json<AuthResponse>, AppError> {
    debug!("signin request received");
    let session = state.database.signin_user(input).await.map_err(|error| {
        error!(error = %error, "signin failed");
        internal_error(error)
    })?;
    Ok(Json(AuthResponse {
        user: map_user(&session.user),
        token: session.token,
    }))
}

#[tracing::instrument(skip(state, headers))]
async fn me(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<UserResponse>, AppError> {
    let token = extract_bearer(&headers)?;
    debug!(token_fingerprint = %token_fingerprint(token), "me request received");
    let user = state
        .database
        .authenticate_user(token)
        .await
        .map_err(|error| {
            error!(error = %error, token_fingerprint = %token_fingerprint(token), "me failed");
            internal_error(error)
        })?;
    Ok(Json(map_user(&user)))
}

#[tracing::instrument(skip(state, headers, input))]
async fn update_me(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(input): Json<UpdateProfileInput>,
) -> Result<Json<UserResponse>, AppError> {
    let token = extract_bearer(&headers)?;
    debug!(token_fingerprint = %token_fingerprint(token), "update_me request received");
    let user = state
        .database
        .update_profile(token, input)
        .await
        .map_err(|error| {
            error!(error = %error, token_fingerprint = %token_fingerprint(token), "update_me failed");
            internal_error(error)
        })?;
    Ok(Json(map_user(&user)))
}

#[tracing::instrument(skip(state, headers))]
async fn list_providers(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<ProviderResponse>>, AppError> {
    let token = extract_bearer(&headers)?;
    debug!(token_fingerprint = %token_fingerprint(token), "list_providers request received");
    let providers = state
        .database
        .list_provider_credentials(token)
        .await
        .map_err(|error| {
            error!(error = %error, token_fingerprint = %token_fingerprint(token), "list_providers failed");
            internal_error(error)
        })?;
    Ok(Json(providers.iter().map(map_provider).collect()))
}

#[tracing::instrument(skip(state, headers, input))]
async fn create_provider(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(input): Json<CreateProviderRequest>,
) -> Result<Json<ProviderResponse>, AppError> {
    let token = extract_bearer(&headers)?;
    debug!(
        token_fingerprint = %token_fingerprint(token),
        provider = %input.provider.as_str(),
        "create_provider request received"
    );
    let provider = state
        .database
        .create_provider_credential(
            token,
            CreateProviderCredentialInput {
                provider: input.provider,
                label: input.label,
                api_key: input.api_key,
                tags: input.tags,
            },
        )
        .await
        .map_err(|error| {
            error!(
                error = %error,
                token_fingerprint = %token_fingerprint(token),
                "create_provider failed"
            );
            internal_error(error)
        })?;
    Ok(Json(map_provider(&provider)))
}

#[tracing::instrument(skip(state, headers))]
async fn get_provider(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(provider_id): Path<String>,
) -> Result<Json<ProviderResponse>, AppError> {
    let token = extract_bearer(&headers)?;
    debug!(
        token_fingerprint = %token_fingerprint(token),
        provider_id = %provider_id,
        "get_provider request received"
    );
    let provider = state
        .database
        .get_provider_credential(token, &provider_id)
        .await
        .map_err(|error| {
            error!(
                error = %error,
                token_fingerprint = %token_fingerprint(token),
                provider_id = %provider_id,
                "get_provider failed"
            );
            internal_error(error)
        })?
        .ok_or_else(|| AppError::NotFound("Provider not found".into()))?;
    Ok(Json(map_provider(&provider)))
}

#[tracing::instrument(skip(state, headers, input))]
async fn update_provider(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(provider_id): Path<String>,
    Json(input): Json<UpdateProviderRequest>,
) -> Result<Json<ProviderResponse>, AppError> {
    let token = extract_bearer(&headers)?;
    debug!(
        token_fingerprint = %token_fingerprint(token),
        provider_id = %provider_id,
        "update_provider request received"
    );
    let provider = state
        .database
        .update_provider_credential(
            token,
            &provider_id,
            UpdateProviderCredentialInput {
                label: input.label,
                api_key: input.api_key,
                tags: input.tags,
                enabled: input.enabled,
            },
        )
        .await
        .map_err(|error| {
            error!(
                error = %error,
                token_fingerprint = %token_fingerprint(token),
                provider_id = %provider_id,
                "update_provider failed"
            );
            internal_error(error)
        })?
        .ok_or_else(|| AppError::NotFound("Provider not found".into()))?;
    Ok(Json(map_provider(&provider)))
}

#[tracing::instrument(skip(state, headers))]
async fn delete_provider(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(provider_id): Path<String>,
) -> Result<Json<ProviderResponse>, AppError> {
    let token = extract_bearer(&headers)?;
    debug!(
        token_fingerprint = %token_fingerprint(token),
        provider_id = %provider_id,
        "delete_provider request received"
    );
    let provider = state
        .database
        .delete_provider_credential(token, &provider_id)
        .await
        .map_err(|error| {
            error!(
                error = %error,
                token_fingerprint = %token_fingerprint(token),
                provider_id = %provider_id,
                "delete_provider failed"
            );
            internal_error(error)
        })?
        .ok_or_else(|| AppError::NotFound("Provider not found".into()))?;
    Ok(Json(map_provider(&provider)))
}

#[tracing::instrument(skip(state, headers))]
async fn list_virtual_keys(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<VirtualKeyResponse>>, AppError> {
    let token = extract_bearer(&headers)?;
    debug!(token_fingerprint = %token_fingerprint(token), "list_virtual_keys request received");
    let keys = state
        .database
        .list_virtual_api_keys(token)
        .await
        .map_err(|error| {
            error!(error = %error, token_fingerprint = %token_fingerprint(token), "list_virtual_keys failed");
            internal_error(error)
        })?;
    Ok(Json(keys.iter().map(map_virtual_key).collect()))
}

#[tracing::instrument(skip(state, headers, input))]
async fn create_virtual_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(input): Json<CreateVirtualKeyRequest>,
) -> Result<Json<CreateVirtualKeyResponse>, AppError> {
    let token = extract_bearer(&headers)?;
    debug!(token_fingerprint = %token_fingerprint(token), "create_virtual_key request received");
    let created = state
        .database
        .create_virtual_api_key(
            token,
            CreateVirtualApiKeyInput {
                name: input.name,
                allowed_models: input.allowed_models,
                tags: input.tags,
                expires_at: input.expires_at,
            },
        )
        .await
        .map_err(|error| {
            error!(error = %error, token_fingerprint = %token_fingerprint(token), "create_virtual_key failed");
            internal_error(error)
        })?;
    Ok(Json(CreateVirtualKeyResponse {
        key: map_virtual_key(&created.record),
        raw_key: created.raw_key,
    }))
}

#[tracing::instrument(skip(state, headers))]
async fn get_virtual_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(key_id): Path<String>,
) -> Result<Json<VirtualKeyResponse>, AppError> {
    let token = extract_bearer(&headers)?;
    debug!(token_fingerprint = %token_fingerprint(token), key_id = %key_id, "get_virtual_key request received");
    let key = state
        .database
        .get_virtual_api_key(token, &key_id)
        .await
        .map_err(|error| {
            error!(error = %error, token_fingerprint = %token_fingerprint(token), key_id = %key_id, "get_virtual_key failed");
            internal_error(error)
        })?
        .ok_or_else(|| AppError::NotFound("Virtual key not found".into()))?;
    Ok(Json(map_virtual_key(&key)))
}

#[tracing::instrument(skip(state, headers, input))]
async fn update_virtual_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(key_id): Path<String>,
    Json(input): Json<UpdateVirtualKeyRequest>,
) -> Result<Json<VirtualKeyResponse>, AppError> {
    let token = extract_bearer(&headers)?;
    debug!(token_fingerprint = %token_fingerprint(token), key_id = %key_id, "update_virtual_key request received");
    let key = state
        .database
        .update_virtual_api_key(
            token,
            &key_id,
            UpdateVirtualApiKeyInput {
                name: input.name,
                allowed_models: input.allowed_models,
                tags: input.tags,
                enabled: input.enabled,
                expires_at: input.expires_at,
            },
        )
        .await
        .map_err(|error| {
            error!(error = %error, token_fingerprint = %token_fingerprint(token), key_id = %key_id, "update_virtual_key failed");
            internal_error(error)
        })?
        .ok_or_else(|| AppError::NotFound("Virtual key not found".into()))?;
    Ok(Json(map_virtual_key(&key)))
}

#[tracing::instrument(skip(state, headers))]
async fn delete_virtual_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(key_id): Path<String>,
) -> Result<Json<VirtualKeyResponse>, AppError> {
    let token = extract_bearer(&headers)?;
    debug!(token_fingerprint = %token_fingerprint(token), key_id = %key_id, "delete_virtual_key request received");
    let key = state
        .database
        .delete_virtual_api_key(token, &key_id)
        .await
        .map_err(|error| {
            error!(error = %error, token_fingerprint = %token_fingerprint(token), key_id = %key_id, "delete_virtual_key failed");
            internal_error(error)
        })?
        .ok_or_else(|| AppError::NotFound("Virtual key not found".into()))?;
    Ok(Json(map_virtual_key(&key)))
}

#[tracing::instrument(skip(state, headers))]
async fn list_models(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<ModelResponse>>, AppError> {
    let token = extract_bearer(&headers)?;
    debug!(token_fingerprint = %token_fingerprint(token), "list_models request received");
    let models = state
        .database
        .list_usable_models(token)
        .await
        .map_err(|error| {
            error!(error = %error, token_fingerprint = %token_fingerprint(token), "list_models failed");
            internal_error(error)
        })?;
    Ok(Json(models.iter().map(map_model).collect()))
}

#[tracing::instrument(skip(state, headers))]
async fn get_model(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(alias): Path<String>,
) -> Result<Json<ModelResponse>, AppError> {
    let token = extract_bearer(&headers)?;
    debug!(token_fingerprint = %token_fingerprint(token), alias = %alias, "get_model request received");
    let model = state
        .database
        .get_model_by_alias_for_user(token, &alias)
        .await
        .map_err(|error| {
            error!(error = %error, token_fingerprint = %token_fingerprint(token), alias = %alias, "get_model failed");
            internal_error(error)
        })?
        .ok_or_else(|| AppError::NotFound("Model not found".into()))?;
    Ok(Json(map_model(&model)))
}

#[tracing::instrument(skip(state, headers, payload))]
async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Response, AppError> {
    let started_at = Instant::now();
    let request_id = Uuid::new_v4().to_string();
    let raw_key = extract_bearer(&headers)?;
    info!(
        %request_id,
        token_fingerprint = %token_fingerprint(raw_key),
        "chat_completions request received"
    );
    payload
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::BadRequest("`model` is required".into()))?;
    let response = proxy::ProxyExecutor::execute_chat_completion(
        state,
        raw_key,
        payload,
        request_id.clone(),
        started_at,
    )
    .await;
    match &response {
        Ok(_) => info!(
            %request_id,
            duration_ms = started_at.elapsed().as_millis(),
            "chat_completions completed"
        ),
        Err(error) => error!(
            %request_id,
            duration_ms = started_at.elapsed().as_millis(),
            error = %error,
            "chat_completions failed"
        ),
    }
    response
}

fn extract_bearer(headers: &HeaderMap) -> Result<&str, AppError> {
    let header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            warn!("missing authorization header");
            AppError::Unauthorized("Missing Authorization header".into())
        })?;

    header.strip_prefix("Bearer ").ok_or_else(|| {
        warn!("authorization header is not bearer");
        AppError::Unauthorized("Authorization header must use Bearer auth".into())
    })
}

fn internal_error(error: impl std::fmt::Display) -> AppError {
    error!(error = %error, "internal handler error");
    AppError::Internal(anyhow::anyhow!(error.to_string()))
}

fn token_fingerprint(token: &str) -> String {
    let prefix: String = token.chars().take(8).collect();
    format!("{prefix}..{}", token.len())
}

fn map_user(user: &User) -> UserResponse {
    UserResponse {
        id: user.id.to_sql(),
        name: user.name.clone(),
        email: user.email.clone(),
        enabled: user.enabled,
    }
}

fn map_provider(provider: &ProviderCredential) -> ProviderResponse {
    ProviderResponse {
        id: provider.id.to_sql(),
        provider: provider.provider.clone(),
        label: provider.label.clone(),
        tags: provider.tags.clone(),
        enabled: provider.enabled,
        last_used_at: provider.last_used_at.map(|value| value.to_rfc3339()),
        created_at: provider.created_at.to_rfc3339(),
        updated_at: provider.updated_at.to_rfc3339(),
    }
}

fn map_virtual_key(key: &VirtualApiKey) -> VirtualKeyResponse {
    VirtualKeyResponse {
        id: key.id.to_sql(),
        name: key.name.clone(),
        key_prefix: key.key_prefix.clone(),
        allowed_models: key.allowed_models.clone(),
        tags: key.tags.clone(),
        enabled: key.enabled,
        expires_at: key.expires_at.map(|value| value.to_rfc3339()),
        last_used_at: key.last_used_at.map(|value| value.to_rfc3339()),
        created_at: key.created_at.to_rfc3339(),
        updated_at: key.updated_at.to_rfc3339(),
    }
}

fn map_model(model: &ModelCatalogEntry) -> ModelResponse {
    ModelResponse {
        id: model.id.to_sql(),
        alias: model.alias.clone(),
        display_name: model.display_name.clone(),
        provider: model.provider.clone(),
        upstream_model: model.upstream_model.clone(),
        description: model.description.clone(),
        tags: model.tags.clone(),
        enabled: model.enabled,
        context_window_tokens: model.context_window_tokens,
        max_output_tokens: model.max_output_tokens,
        supports_streaming: model.supports_streaming,
        supports_thinking: model.supports_thinking,
        thinking_required: model.thinking_required,
        supports_temperature: model.supports_temperature,
        temperature_fixed_to: model.temperature_fixed_to,
        temperature_min: model.temperature_min,
        temperature_max: model.temperature_max,
        supports_top_p: model.supports_top_p,
        supports_system_messages: model.supports_system_messages,
        supports_tools: model.supports_tools,
        supports_vision: model.supports_vision,
        supports_json_mode: model.supports_json_mode,
        supports_parallel_tool_calls: model.supports_parallel_tool_calls,
    }
}
