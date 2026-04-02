mod google_genai;
mod types;

use std::{sync::Arc, time::Instant};

use axum::{
    Json,
    body::Body,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::Value;
use tracing::{info, warn};
use valymux_core::error::AppError;
use valymux_surrealdb::{DatabaseError, ProxySession, RequestLogInput, ResolvedProxyRoute};

use crate::sys::state::AppState;

pub use types::CanonicalChatRequest;

pub struct ProxyExecutor;

struct RequestLogDraft<'a> {
    request_id: &'a str,
    route: &'a ResolvedProxyRoute,
    status_code: i64,
    latency_ms: i64,
    request_url: &'a str,
    stream: bool,
    error_message: Option<String>,
    usage: UsageSummary,
}

impl ProxyExecutor {
    /// # Errors
    /// Returns an error when request validation, route resolution, upstream calls, or response
    /// translation fails.
    pub async fn execute_chat_completion(
        state: Arc<AppState>,
        raw_key: &str,
        payload: Value,
        request_id: String,
        started_at: Instant,
    ) -> Result<Response, AppError> {
        let canonical_request = CanonicalChatRequest::from_value(payload)
            .map_err(|error| AppError::BadRequest(error.to_string()))?;
        let session = state
            .database
            .begin_proxy_session(raw_key)
            .await
            .map_err(map_proxy_database_error)?;
        let route = session
            .resolve_route(&canonical_request.model)
            .await
            .map_err(map_proxy_database_error)?;

        let mut canonical_request = canonical_request;
        canonical_request.temperature = validate_model_capabilities(&canonical_request, &route)
            .map_err(|error| AppError::BadRequest(error.to_string()))?;

        let provider_api_key = state
            .database
            .fetch_proxy_provider_api_key(&route.provider_credential_id)
            .await
            .map_err(map_proxy_database_error)?;
        let adapter = provider_adapter(&route.model.provider)?;
        adapter
            .validate_request(&canonical_request, &route)
            .map_err(|error| AppError::BadRequest(error.to_string()))?;

        let request_url = adapter.request_url(&route, canonical_request.stream);
        let outbound_payload = adapter
            .prepare_body(&canonical_request, &route)
            .map_err(|error| AppError::BadRequest(error.to_string()))?;
        let builder = adapter
            .apply_headers(
                state.reqwest_client.post(&request_url),
                &provider_api_key,
                &request_id,
            )
            .json(&outbound_payload);
        let request = builder.build().map_err(|e| AppError::Internal(e.into()))?;
        let upstream = state
            .http_client
            .execute(request)
            .await
            .map_err(|error| AppError::Internal(error.into()))?;

        let status_code = i64::from(upstream.status().as_u16());
        let ms = started_at.elapsed().as_millis();
        let latency_ms = i64::try_from(ms).unwrap_or(i64::MAX);

        if canonical_request.stream {
            if let Some(headers) = adapter.stream_headers() {
                let stream_body = adapter
                    .translate_stream_response(upstream, &canonical_request, &route, &request_id)
                    .map_err(internal_error)?;
                persist_request_log_or_warn(
                    &session,
                    build_request_log(RequestLogDraft {
                        request_id: &request_id,
                        route: &route,
                        status_code,
                        latency_ms,
                        request_url: &request_url,
                        stream: true,
                        error_message: None,
                        usage: UsageSummary {
                            prompt_tokens: None,
                            completion_tokens: None,
                        },
                    }),
                )
                .await;

                let response_status = upstream_status_code_or_warn(status_code, &request_url);
                return Ok((response_status, headers, stream_body).into_response());
            }

            return Err(AppError::BadRequest(format!(
                "{} streaming translation is not implemented yet",
                adapter.name()
            )));
        }

        let body = upstream
            .json::<Value>()
            .await
            .map_err(|error| AppError::Internal(error.into()))?;
        let usage = adapter.extract_usage(&body);
        let translated = adapter.translate_json_response(&body, &canonical_request, &route);

        persist_request_log_or_warn(
            &session,
            build_request_log(RequestLogDraft {
                request_id: &request_id,
                route: &route,
                status_code,
                latency_ms,
                request_url: &request_url,
                stream: false,
                error_message: if status_code >= 400 {
                    adapter.error_message(&body)
                } else {
                    None
                },
                usage,
            }),
        )
        .await;

        let response_status = upstream_status_code_or_warn(status_code, &request_url);
        Ok((response_status, Json(translated)).into_response())
    }
}

fn validate_model_capabilities(
    request: &CanonicalChatRequest,
    route: &ResolvedProxyRoute,
) -> anyhow::Result<Option<f64>> {
    const FLOAT_TOLERANCE: f64 = f64::EPSILON;

    if request.stream && !route.model.supports_streaming {
        anyhow::bail!("model does not support streaming");
    }

    if request.temperature.is_some() && !route.model.supports_temperature {
        anyhow::bail!("model does not support temperature");
    }

    let enforced_temperature = if let Some(fixed) = route.model.temperature_fixed_to {
        match request.temperature {
            Some(requested) if (requested - fixed).abs() > FLOAT_TOLERANCE => {
                anyhow::bail!("temperature must be {fixed} for this model");
            }
            _ => Some(fixed),
        }
    } else {
        request.temperature
    };

    if let Some(temperature) = enforced_temperature {
        if let Some(minimum) = route.model.temperature_min
            && temperature < minimum
        {
            anyhow::bail!("temperature must be at least {minimum} for this model");
        }

        if let Some(maximum) = route.model.temperature_max
            && temperature > maximum
        {
            anyhow::bail!("temperature must be at most {maximum} for this model");
        }
    }

    if request.top_p.is_some() && !route.model.supports_top_p {
        anyhow::bail!("model does not support top_p");
    }

    if request.tools.is_some() && !route.model.supports_tools {
        anyhow::bail!("model does not support tools");
    }

    Ok(enforced_temperature)
}

fn build_request_log(draft: RequestLogDraft<'_>) -> RequestLogInput {
    RequestLogInput {
        request_id: draft.request_id.to_string(),
        model_alias: draft.route.model.alias.clone(),
        provider: draft.route.provider.clone(),
        upstream_model: draft.route.model.upstream_model.clone(),
        status_code: draft.status_code,
        latency_ms: draft.latency_ms,
        stream: draft.stream,
        request_url: draft.request_url.to_string(),
        error_message: draft.error_message,
        usage_input_tokens: draft.usage.prompt_tokens,
        usage_output_tokens: draft.usage.completion_tokens,
    }
}

fn provider_adapter(provider: &str) -> Result<&'static dyn ProviderAdapter, AppError> {
    match provider {
        "google-genai" => Ok(&google_genai::GoogleGenAiAdapter),
        _ => Err(AppError::BadRequest(format!(
            "Unsupported provider `{provider}`"
        ))),
    }
}

pub struct UsageSummary {
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
}

pub trait ProviderAdapter: Sync {
    fn name(&self) -> &'static str;
    fn target_url(&self) -> &'static str;
    /// Returns the full request URL for the upstream call. Defaults to `target_url()`, but
    /// providers whose URL embeds the model name or operation (e.g. Google GenAI) should
    /// override this.
    fn request_url(&self, route: &ResolvedProxyRoute, stream: bool) -> String {
        let _ = (route, stream);
        self.target_url().to_string()
    }
    /// # Errors
    /// Returns an error when provider-specific constraints are not met for the request or route.
    fn validate_request(
        &self,
        request: &CanonicalChatRequest,
        route: &ResolvedProxyRoute,
    ) -> anyhow::Result<()>;
    /// # Errors
    /// Returns an error when the canonical request cannot be translated into a valid provider
    /// payload.
    fn prepare_body(
        &self,
        request: &CanonicalChatRequest,
        route: &ResolvedProxyRoute,
    ) -> anyhow::Result<Value>;
    fn apply_headers(
        &self,
        builder: reqwest::RequestBuilder,
        api_key: &str,
        request_id: &str,
    ) -> reqwest::RequestBuilder;
    fn translate_json_response(
        &self,
        body: &Value,
        request: &CanonicalChatRequest,
        route: &ResolvedProxyRoute,
    ) -> Value;
    /// # Errors
    /// Returns an error when the provider stream cannot be translated into OpenAI-compatible SSE
    /// chunks.
    fn translate_stream_response(
        &self,
        upstream: reqwest::Response,
        request: &CanonicalChatRequest,
        route: &ResolvedProxyRoute,
        request_id: &str,
    ) -> anyhow::Result<Body>;
    fn extract_usage(&self, body: &Value) -> UsageSummary;
    fn error_message(&self, body: &Value) -> Option<String>;
    fn stream_headers(&self) -> Option<HeaderMap> {
        None
    }
}

pub(crate) fn openai_stream_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream"),
    );
    headers.insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache"),
    );
    headers.insert(
        axum::http::header::CONNECTION,
        HeaderValue::from_static("keep-alive"),
    );
    headers
}

fn internal_error(error: impl std::fmt::Display) -> AppError {
    AppError::Internal(anyhow::anyhow!(error.to_string()))
}

fn map_proxy_database_error(error: DatabaseError) -> AppError {
    match error {
        DatabaseError::NotFound(msg) => AppError::NotFound(msg),
        DatabaseError::InvalidConfig(msg) => AppError::Unauthorized(msg),
        DatabaseError::ServiceAuth(msg) => AppError::Unauthorized(msg),
        DatabaseError::SecretFetch(inner) => internal_error(inner),
        DatabaseError::Database(inner) => classify_proxy_database_message(inner.to_string()),
        other => internal_error(other),
    }
}

fn classify_proxy_database_message(message: String) -> AppError {
    let lower = message.to_lowercase();

    if lower.contains("invalid or expired virtual api key")
        || lower.contains("no record was returned")
        || lower.contains("invalidtoken")
    {
        return AppError::Unauthorized("Invalid or expired virtual API key".into());
    }

    if lower.contains("not allowed to use this model") {
        return AppError::BadRequest(message);
    }

    if lower.contains("requested model not found in catalog")
        || lower.contains("provider credential is disabled or not found")
        || lower.contains("provider credential not found or disabled")
    {
        return AppError::NotFound(message);
    }

    internal_error(message)
}

async fn persist_request_log_or_warn(session: &ProxySession, log: RequestLogInput) {
    let request_id = log.request_id.clone();
    let model_alias = log.model_alias.clone();
    let status_code = log.status_code;

    if let Err(error) = session.log_request(log).await {
        warn!(
            %request_id,
            %model_alias,
            status_code,
            error = %error,
            "failed to persist request log",
        );
    } else {
        info!(%request_id, %model_alias, status_code, "persisted request log");
    }
}

fn upstream_status_code_or_warn(status_code: i64, upstream: &str) -> StatusCode {
    if let Some(status) = u16::try_from(status_code)
        .ok()
        .and_then(|status| StatusCode::from_u16(status).ok())
    {
        status
    } else {
        warn!(
            status_code,
            upstream, "invalid upstream status code, falling back to 502"
        );
        StatusCode::BAD_GATEWAY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_virtual_key_auth_failures_as_unauthorized() {
        let error = classify_proxy_database_message("No record was returned".into());
        assert!(matches!(error, AppError::Unauthorized(_)));
    }

    #[test]
    fn classifies_scoped_model_rejections_as_bad_request() {
        let error = classify_proxy_database_message(
            "An error occurred: virtual API key is not allowed to use this model".into(),
        );
        assert!(matches!(error, AppError::BadRequest(_)));
    }
}
