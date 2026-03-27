use anyhow::{Result, bail};
use axum::{body::Body, http::HeaderMap};
use serde_json::{Value, json};

use super::{CanonicalChatRequest, ProviderAdapter, UsageSummary, openai_stream_headers};
use valymux_surrealdb::ResolvedProxyRoute;

pub struct OpenAiAdapter;

const OPENAI_ALLOWED_PROVIDER_KEYS: &[&str] = &["service_tier", "reasoning"];

impl ProviderAdapter for OpenAiAdapter {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn target_url(&self) -> &'static str {
        "https://api.openai.com/v1/chat/completions"
    }

    fn validate_request(
        &self,
        request: &CanonicalChatRequest,
        _route: &ResolvedProxyRoute,
    ) -> Result<()> {
        if let Some(provider_options) = request.provider_options_for(self.name())
            && provider_options
                .keys()
                .any(|key| key.as_str() != "service_tier" && key.as_str() != "reasoning")
        {
            bail!("unsupported openai provider option provided");
        }

        Ok(())
    }

    fn prepare_body(
        &self,
        request: &CanonicalChatRequest,
        route: &ResolvedProxyRoute,
    ) -> Result<Value> {
        let mut payload = json!({
            "model": route.model.upstream_model,
            "messages": request.messages,
            "stream": request.stream,
        });

        if let Some(body) = payload.as_object_mut() {
            insert_optional(
                body,
                "temperature",
                request.temperature.map(|value| json!(value)),
            );
            insert_optional(body, "top_p", request.top_p.map(|value| json!(value)));
            insert_optional(
                body,
                "max_tokens",
                request.max_tokens.map(|value| json!(value)),
            );
            insert_optional(
                body,
                "max_completion_tokens",
                request.max_completion_tokens.map(|value| json!(value)),
            );
            insert_optional(body, "stop", request.stop.clone());
            insert_optional(
                body,
                "presence_penalty",
                request.presence_penalty.map(|value| json!(value)),
            );
            insert_optional(
                body,
                "frequency_penalty",
                request.frequency_penalty.map(|value| json!(value)),
            );
            insert_optional(body, "n", request.n.map(|value| json!(value)));
            insert_optional(body, "tools", request.tools.clone());
            insert_optional(body, "response_format", request.response_format.clone());

            for passthrough_key in [
                "tool_choice",
                "parallel_tool_calls",
                "user",
                "metadata",
                "reasoning_effort",
                "modalities",
            ] {
                if let Some(value) = request.raw_body.get(passthrough_key).cloned() {
                    body.insert(passthrough_key.to_string(), value);
                }
            }

            if let Some(provider_options) = request.provider_options_for(self.name()) {
                for (key, value) in provider_options {
                    if OPENAI_ALLOWED_PROVIDER_KEYS.contains(&key.as_str()) {
                        body.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        Ok(payload)
    }

    fn apply_headers(
        &self,
        builder: reqwest::RequestBuilder,
        api_key: &str,
        request_id: &str,
    ) -> reqwest::RequestBuilder {
        builder
            .bearer_auth(api_key)
            .header("x-request-id", request_id)
    }

    fn translate_json_response(
        &self,
        body: &Value,
        _request: &CanonicalChatRequest,
        _route: &ResolvedProxyRoute,
    ) -> Value {
        body.clone()
    }

    fn translate_stream_response(
        &self,
        upstream: reqwest::Response,
        _request: &CanonicalChatRequest,
        _route: &ResolvedProxyRoute,
        _request_id: &str,
    ) -> Result<Body> {
        Ok(Body::from_stream(upstream.bytes_stream()))
    }

    fn extract_usage(&self, body: &Value) -> UsageSummary {
        UsageSummary {
            prompt_tokens: body
                .get("usage")
                .and_then(|usage| usage.get("prompt_tokens"))
                .and_then(Value::as_i64),
            completion_tokens: body
                .get("usage")
                .and_then(|usage| usage.get("completion_tokens"))
                .and_then(Value::as_i64),
        }
    }

    fn error_message(&self, body: &Value) -> Option<String> {
        body.get("error")
            .and_then(|error| error.get("message"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
    }

    fn stream_headers(&self) -> Option<HeaderMap> {
        Some(openai_stream_headers())
    }
}

fn insert_optional(body: &mut serde_json::Map<String, Value>, key: &str, value: Option<Value>) {
    if let Some(value) = value {
        body.insert(key.to_string(), value);
    }
}
