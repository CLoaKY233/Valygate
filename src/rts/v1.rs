use sha2::{Digest, Sha256};
use std::{sync::Arc, time::Instant};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb_types::{RecordId, ToSql};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use valymux_core::error::AppError;
use valymux_surrealdb::{
    CreateProviderCredentialInput, CreateVirtualApiKeyInput, DatabaseError, ModelDefinition,
    ProviderCredential, ProviderKind, SigninInput, SignupInput, UpdateProfileInput,
    UpdateProviderCredentialInput, UpdateVirtualApiKeyInput, User, VirtualApiKey,
    VirtualKeyRouteInput,
};

use crate::{rts::extractors::RequireAuth, svc::proxy, sys::state::AppState};

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
        .route("/providers/{provider_id}/sync", post(sync_provider))
        .route("/providers/{provider_id}/models", get(list_provider_models))
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
        .route("/models/{*alias}", get(get_model))
        .route("/v1/chat/completions", post(chat_completions))
}

// ── Response types ────────────────────────────────────────────────────────────

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
    sync_status: String,
    sync_error: Option<String>,
    last_synced_at: Option<String>,
    model_count: i64,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct VirtualKeyResponse {
    id: String,
    name: String,
    key_prefix: String,
    allowed_models: Vec<String>,
    model_routes: Vec<VirtualKeyRouteResponse>,
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
#[allow(clippy::struct_excessive_bools)]
struct ModelResponse {
    id: String,
    alias: String,
    display_name: String,
    provider: String,
    upstream_model: String,
    description: Option<String>,
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

// ── Request types ─────────────────────────────────────────────────────────────

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
    model_routes: Vec<VirtualKeyRouteRequest>,
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
    model_routes: Vec<VirtualKeyRouteRequest>,
    #[serde(default)]
    tags: Vec<String>,
    enabled: bool,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize)]
struct VirtualKeyRouteRequest {
    model_alias: String,
    provider_credential_id: String,
}

#[derive(Serialize)]
struct VirtualKeyRouteResponse {
    model_alias: String,
    provider_credential_id: String,
    provider: String,
    provider_label: String,
}

// ── Auth handlers ─────────────────────────────────────────────────────────────

#[tracing::instrument(skip(state, input))]
async fn signup(
    State(state): State<Arc<AppState>>,
    Json(input): Json<SignupInput>,
) -> Result<Json<AuthResponse>, AppError> {
    debug!("signup request received");
    let session = state.database.signup_user(input).await.map_err(|error| {
        error!(error = %error, "signup failed");
        match error {
            DatabaseError::InvalidConfig(msg) => AppError::BadRequest(msg),
            other => internal_error(other),
        }
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
        match error {
            DatabaseError::InvalidConfig(msg) => AppError::Unauthorized(msg),
            other => internal_error(other),
        }
    })?;
    Ok(Json(AuthResponse {
        user: map_user(&session.user),
        token: session.token,
    }))
}

// ── Profile handlers ──────────────────────────────────────────────────────────

#[tracing::instrument(skip(auth))]
async fn me(auth: RequireAuth) -> Result<Json<UserResponse>, AppError> {
    let token = &auth.token;
    debug!(token_fingerprint = %token_fingerprint(token), "me request received");
    Ok(Json(map_user(&auth.user)))
}

#[tracing::instrument(skip(state, auth, input))]
async fn update_me(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Json(input): Json<UpdateProfileInput>,
) -> Result<Json<UserResponse>, AppError> {
    let token = &auth.token;
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

// ── Provider credential handlers ──────────────────────────────────────────────

#[tracing::instrument(skip(state, auth))]
async fn list_providers(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
) -> Result<Json<Vec<ProviderResponse>>, AppError> {
    let token = &auth.token;
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

#[tracing::instrument(skip(state, auth, input))]
async fn create_provider(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Json(input): Json<CreateProviderRequest>,
) -> Result<Json<ProviderResponse>, AppError> {
    let token = &auth.token;
    debug!(
        token_fingerprint = %token_fingerprint(token),
        provider = %input.provider.as_str(),
        "create_provider request received"
    );
    let credential = state
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

    // Spawn background model discovery — does not block the response
    let state_clone = Arc::clone(&state);
    let token_clone = token.to_string();
    let credential_id = credential.id.to_sql();
    tokio::spawn(async move {
        crate::svc::discovery::run_sync(state_clone, &token_clone, &credential_id).await;
    });

    Ok(Json(map_provider(&credential)))
}

#[tracing::instrument(skip(state, auth))]
async fn get_provider(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(provider_id): Path<String>,
) -> Result<Json<ProviderResponse>, AppError> {
    let token = &auth.token;
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
            map_database_error(error)
        })?
        .ok_or_else(|| AppError::NotFound("Provider not found".into()))?;
    Ok(Json(map_provider(&provider)))
}

#[tracing::instrument(skip(state, auth, input))]
async fn update_provider(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(provider_id): Path<String>,
    Json(input): Json<UpdateProviderRequest>,
) -> Result<Json<ProviderResponse>, AppError> {
    let token = &auth.token;
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
            map_database_error(error)
        })?
        .ok_or_else(|| AppError::NotFound("Provider not found".into()))?;
    Ok(Json(map_provider(&provider)))
}

#[tracing::instrument(skip(state, auth))]
async fn delete_provider(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(provider_id): Path<String>,
) -> Result<Json<ProviderResponse>, AppError> {
    let token = &auth.token;
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

/// Triggers a manual model re-sync for a provider credential. Returns 202 immediately.
#[tracing::instrument(skip(state, auth))]
async fn sync_provider(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(provider_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let token = &auth.token;
    debug!(
        token_fingerprint = %token_fingerprint(token),
        provider_id = %provider_id,
        "sync_provider request received"
    );

    // Verify user owns this credential
    let credential = state
        .database
        .get_provider_credential(token, &provider_id)
        .await
        .map_err(|error| {
            error!(
                error = %error,
                token_fingerprint = %token_fingerprint(token),
                provider_id = %provider_id,
                "sync_provider: get_provider_credential failed"
            );
            internal_error(error)
        })?
        .ok_or_else(|| AppError::NotFound("Provider not found".into()))?;

    let credential_id = credential.id.to_sql();

    // Mark as syncing immediately
    state
        .database
        .set_credential_sync_status(token, &credential_id, "syncing", None)
        .await
        .map_err(|error| {
            error!(error = %error, credential_id = %credential_id, "sync_provider: failed to set status");
            internal_error(error)
        })?;

    // Spawn background task - need to clone token for the async task
    let state_clone = Arc::clone(&state);
    let token_clone = token.to_string();
    let cid = credential_id.clone();
    tokio::spawn(async move {
        crate::svc::discovery::run_sync(state_clone, &token_clone, &cid).await;
    });

    Ok(StatusCode::ACCEPTED)
}

/// Returns all discovered models for a specific provider credential.
#[tracing::instrument(skip(state, auth))]
async fn list_provider_models(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(provider_id): Path<String>,
) -> Result<Json<Vec<ModelResponse>>, AppError> {
    let token = &auth.token;
    debug!(
        token_fingerprint = %token_fingerprint(token),
        provider_id = %provider_id,
        "list_provider_models request received"
    );

    // Verify user owns this credential (row-level security enforced by get_provider_credential)
    state
        .database
        .get_provider_credential(token, &provider_id)
        .await
        .map_err(|error| {
            error!(
                error = %error,
                token_fingerprint = %token_fingerprint(token),
                provider_id = %provider_id,
                "list_provider_models: ownership check failed"
            );
            internal_error(error)
        })?
        .ok_or_else(|| AppError::NotFound("Provider not found".into()))?;

    let models = state
        .database
        .list_models_for_credential(token, &provider_id)
        .await
        .map_err(|error| {
            error!(
                error = %error,
                token_fingerprint = %token_fingerprint(token),
                provider_id = %provider_id,
                "list_provider_models failed"
            );
            internal_error(error)
        })?;

    Ok(Json(models.iter().map(map_model).collect()))
}

// ── Virtual key handlers ──────────────────────────────────────────────────────

#[tracing::instrument(skip(state, auth))]
async fn list_virtual_keys(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
) -> Result<Json<Vec<VirtualKeyResponse>>, AppError> {
    let token = &auth.token;
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

#[tracing::instrument(skip(state, auth, input))]
async fn create_virtual_key(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Json(input): Json<CreateVirtualKeyRequest>,
) -> Result<Json<CreateVirtualKeyResponse>, AppError> {
    let token = &auth.token;
    debug!(token_fingerprint = %token_fingerprint(token), "create_virtual_key request received");
    let routes = parse_virtual_key_routes(input.model_routes)?;
    let created = state
        .database
        .create_virtual_api_key(
            token,
            CreateVirtualApiKeyInput {
                name: input.name,
                allowed_models: input.allowed_models,
                routes,
                tags: input.tags,
                expires_at: input.expires_at,
            },
        )
        .await
        .map_err(|error| {
            error!(error = %error, token_fingerprint = %token_fingerprint(token), "create_virtual_key failed");
            map_database_error(error)
        })?;
    Ok(Json(CreateVirtualKeyResponse {
        key: map_virtual_key(&created.record),
        raw_key: created.raw_key,
    }))
}

#[tracing::instrument(skip(state, auth))]
async fn get_virtual_key(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(key_id): Path<String>,
) -> Result<Json<VirtualKeyResponse>, AppError> {
    let token = &auth.token;
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

#[tracing::instrument(skip(state, auth, input))]
async fn update_virtual_key(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(key_id): Path<String>,
    Json(input): Json<UpdateVirtualKeyRequest>,
) -> Result<Json<VirtualKeyResponse>, AppError> {
    let token = &auth.token;
    debug!(token_fingerprint = %token_fingerprint(token), key_id = %key_id, "update_virtual_key request received");
    let routes = parse_virtual_key_routes(input.model_routes)?;
    let key = state
        .database
        .update_virtual_api_key(
            token,
            &key_id,
            UpdateVirtualApiKeyInput {
                name: input.name,
                allowed_models: input.allowed_models,
                routes,
                tags: input.tags,
                enabled: input.enabled,
                expires_at: input.expires_at,
            },
        )
        .await
        .map_err(|error| {
            error!(error = %error, token_fingerprint = %token_fingerprint(token), key_id = %key_id, "update_virtual_key failed");
            map_database_error(error)
        })?
        .ok_or_else(|| AppError::NotFound("Virtual key not found".into()))?;
    Ok(Json(map_virtual_key(&key)))
}

#[tracing::instrument(skip(state, auth))]
async fn delete_virtual_key(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(key_id): Path<String>,
) -> Result<Json<VirtualKeyResponse>, AppError> {
    let token = &auth.token;
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

// ── Model catalog handlers ────────────────────────────────────────────────────

#[tracing::instrument(skip(state, auth))]
async fn list_models(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
) -> Result<Json<Vec<ModelResponse>>, AppError> {
    let token = &auth.token;
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

#[tracing::instrument(skip(state, auth))]
async fn get_model(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(alias): Path<String>,
) -> Result<Json<ModelResponse>, AppError> {
    let alias = alias.trim_start_matches('/').to_string();
    let token = &auth.token;
    debug!(token_fingerprint = %token_fingerprint(token), alias = %alias, "get_model request received");
    let model = state
        .database
        .get_model_by_alias(token, &alias)
        .await
        .map_err(|error| {
            error!(error = %error, token_fingerprint = %token_fingerprint(token), alias = %alias, "get_model failed");
            internal_error(error)
        })?
        .ok_or_else(|| AppError::NotFound("Model not found".into()))?;
    Ok(Json(map_model(&model)))
}

// ── Proxy handler ─────────────────────────────────────────────────────────────

#[tracing::instrument(skip(state, headers, payload))]
async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
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

// ── Helpers ───────────────────────────────────────────────────────────────────

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

fn map_database_error(error: DatabaseError) -> AppError {
    match error {
        DatabaseError::NotFound(msg) => AppError::NotFound(msg),
        DatabaseError::InvalidConfig(msg) => AppError::Unauthorized(msg),
        DatabaseError::SecretFetch(msg) => AppError::Internal(anyhow::anyhow!(msg)),
        DatabaseError::ServiceAuth(msg) => AppError::Internal(anyhow::anyhow!(msg)),
        DatabaseError::SchemaBootstrap(msg) => AppError::Internal(anyhow::anyhow!(msg)),
        DatabaseError::Crypto(msg) => AppError::Internal(anyhow::anyhow!(msg)),
        DatabaseError::Database(inner) => classify_database_message(inner.to_string()),
    }
}

fn classify_database_message(message: String) -> AppError {
    let lower = message.to_lowercase();

    if lower.contains("invalid or expired virtual api key")
        || lower.contains("no record was returned")
        || lower.contains("invalidtoken")
    {
        return AppError::Unauthorized("Invalid or expired credentials".into());
    }

    if lower.contains("not allowed to use this model") {
        return AppError::BadRequest("Virtual key is not allowed to use this model".into());
    }

    if lower.contains("model alias(es) not found in your catalog") {
        return AppError::BadRequest("One or more model aliases were not found in your catalog".into());
    }

    if lower.contains("virtual api key has no route configured for this model") {
        return AppError::BadRequest("No route configured for this model on this virtual key".into());
    }

    if lower.contains("provider credential does not support model alias")
        || lower.contains("provider credential does not support requested model")
    {
        return AppError::BadRequest(
            "Provider credential does not support the requested model".into(),
        );
    }

    if lower.contains("route alias is outside virtual key scope") {
        return AppError::BadRequest("Route model alias is outside the virtual key's allowed scope".into());
    }

    if lower.contains("expected `record<provider_credential>`") {
        return AppError::BadRequest("Invalid provider credential ID".into());
    }

    if lower.contains("expected `record<virtual_api_key>`") {
        return AppError::BadRequest("Invalid virtual key ID".into());
    }

    if lower.contains("requested model not found in catalog") {
        return AppError::NotFound("Model not found".into());
    }

    if lower.contains("provider credential not found or access denied") {
        return AppError::NotFound("Provider credential not found or access denied".into());
    }

    if lower.contains("provider credential not found or disabled") {
        return AppError::NotFound("Provider credential not found or disabled".into());
    }

    internal_error(message)
}

fn token_fingerprint(token: &str) -> String {
    let hash = Sha256::digest(token.as_bytes());
    hex::encode(&hash[..8])
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
        last_used_at: provider.last_used_at.map(|v| v.to_rfc3339()),
        sync_status: provider.sync_status.clone(),
        sync_error: provider.sync_error.clone(),
        last_synced_at: provider.last_synced_at.map(|v| v.to_rfc3339()),
        model_count: provider.model_count,
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
        model_routes: key
            .routes
            .iter()
            .map(|route| VirtualKeyRouteResponse {
                model_alias: route.model_alias.clone(),
                provider_credential_id: route.provider_credential_id.to_sql(),
                provider: route.provider.clone(),
                provider_label: route.provider_label.clone(),
            })
            .collect(),
        tags: key.tags.clone(),
        enabled: key.enabled,
        expires_at: key.expires_at.map(|v| v.to_rfc3339()),
        last_used_at: key.last_used_at.map(|v| v.to_rfc3339()),
        created_at: key.created_at.to_rfc3339(),
        updated_at: key.updated_at.to_rfc3339(),
    }
}

fn map_model(model: &ModelDefinition) -> ModelResponse {
    ModelResponse {
        id: model.id.to_sql(),
        alias: model.alias.clone(),
        display_name: model.display_name.clone(),
        provider: model.provider.clone(),
        upstream_model: model.upstream_model.clone(),
        description: model.description.clone(),
        enabled: true,
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

fn parse_virtual_key_routes(
    routes: Vec<VirtualKeyRouteRequest>,
) -> Result<Vec<VirtualKeyRouteInput>, AppError> {
    routes
        .into_iter()
        .map(|route| {
            Ok(VirtualKeyRouteInput {
                model_alias: route.model_alias,
                provider_credential_id: RecordId::parse_simple(&route.provider_credential_id)
                    .map_err(|_| {
                        AppError::BadRequest(format!(
                            "invalid provider credential id: {}",
                            route.provider_credential_id
                        ))
                    })?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_virtual_key_scope_violation_as_bad_request() {
        let error = classify_database_message(
            "An error occurred: virtual API key is not allowed to use this model".into(),
        );
        assert!(matches!(error, AppError::BadRequest(_)));
    }

    #[test]
    fn classifies_missing_proxy_auth_record_as_unauthorized() {
        let error = classify_database_message("No record was returned".into());
        assert!(matches!(error, AppError::Unauthorized(_)));
    }

    #[test]
    fn classifies_cross_user_provider_update_as_not_found() {
        let error = classify_database_message(
            "An error occurred: provider credential not found or access denied".into(),
        );
        assert!(matches!(error, AppError::NotFound(_)));
    }

    #[test]
    fn normalizes_wildcard_model_alias_path() {
        let alias = "/google-genai/gemini-2.5-flash".trim_start_matches('/');
        assert_eq!(alias, "google-genai/gemini-2.5-flash");
    }
}
