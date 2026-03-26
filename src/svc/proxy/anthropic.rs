use anyhow::{Result, anyhow, bail};
use async_stream::stream;
use axum::{body::Body, http::HeaderMap};
use bytes::Bytes;
use futures_util::StreamExt;
use serde_json::{Value, json};
use std::io;

use super::{CanonicalChatRequest, ProviderAdapter, UsageSummary, openai_stream_headers};
use valymux_surrealdb::ResolvedProxyRoute;

pub struct AnthropicAdapter;

const ANTHROPIC_RESERVED_PROVIDER_KEYS: &[&str] = &["model", "messages", "max_tokens", "stream"];
const ANTHROPIC_ALLOWED_PROVIDER_KEYS: &[&str] = &["metadata", "stop_sequences", "thinking"];

impl ProviderAdapter for AnthropicAdapter {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    fn target_url(&self) -> &'static str {
        "https://api.anthropic.com/v1/messages"
    }

    fn validate_request(
        &self,
        request: &CanonicalChatRequest,
        _route: &ResolvedProxyRoute,
    ) -> Result<()> {
        validate_provider_options(request.provider_options_for(self.name()))?;
        Ok(())
    }

    fn prepare_body(
        &self,
        request: &CanonicalChatRequest,
        route: &ResolvedProxyRoute,
    ) -> Result<Value> {
        let messages = request
            .raw_body
            .get("messages")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow!("messages array is required"))?;

        let mut system = Vec::new();
        let mut anthropic_messages = Vec::new();

        for message in messages {
            let role = message
                .get("role")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("message role is required"))?;
            let content = message
                .get("content")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    // TODO(VG-ANTH-STRUCTURED-CONTENT): support array/structured content blocks
                    // (images, tool results, vision outputs) instead of string-only MVP handling.
                    anyhow!("only string message content is supported for anthropic in MVP")
                })?;

            match role {
                "system" => system.push(content.to_string()),
                "user" | "assistant" => anthropic_messages.push(json!({
                    "role": role,
                    "content": content,
                })),
                _ => bail!("message role `{role}` is not supported for anthropic in MVP"),
            }
        }

        let max_tokens = request
            .max_completion_tokens
            .or(request.max_tokens)
            .unwrap_or(route.model.max_output_tokens)
            .min(route.model.max_output_tokens);

        let mut body = json!({
            "model": route.model.upstream_model,
            "messages": anthropic_messages,
            "max_tokens": max_tokens,
            "stream": request.stream,
        });

        if !system.is_empty() {
            body["system"] = Value::String(system.join("\n\n"));
        }

        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }

        if let Some(body_object) = body.as_object_mut() {
            merge_provider_options(body_object, request.provider_options_for(self.name()));
        }

        Ok(body)
    }

    fn apply_headers(
        &self,
        builder: reqwest::RequestBuilder,
        api_key: &str,
        request_id: &str,
    ) -> reqwest::RequestBuilder {
        builder
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("x-request-id", request_id)
    }

    fn translate_json_response(
        &self,
        body: &Value,
        _request: &CanonicalChatRequest,
        route: &ResolvedProxyRoute,
    ) -> Value {
        let content = body
            .get("content")
            .and_then(Value::as_array)
            .map(|parts| {
                let text: String = parts
                    .iter()
                    .filter_map(|p| p.get("text").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join("");
                if text.is_empty() {
                    parts
                        .iter()
                        .filter_map(|p| p.get("thinking").and_then(Value::as_str))
                        .collect::<Vec<_>>()
                        .join("")
                } else {
                    text
                }
            })
            .unwrap_or_default();

        let usage = body.get("usage").cloned().unwrap_or_else(|| json!({}));
        let input_tokens = usage.get("input_tokens").and_then(Value::as_i64);
        let output_tokens = usage.get("output_tokens").and_then(Value::as_i64);

        json!({
            "id": body.get("id").cloned().unwrap_or_else(|| json!("unknown")),
            "object": "chat.completion",
            "created": chrono::Utc::now().timestamp(),
            "model": route.model.alias,
            "provider_model": route.model.upstream_model,
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": content,
                },
                "finish_reason": map_finish_reason(body.get("stop_reason")),
            }],
            "usage": {
                "prompt_tokens": input_tokens,
                "completion_tokens": output_tokens,
                "total_tokens": match (input_tokens, output_tokens) {
                    (Some(input), Some(output)) => Some(input + output),
                    _ => None,
                },
            },
        })
    }

    fn translate_stream_response(
        &self,
        upstream: reqwest::Response,
        _request: &CanonicalChatRequest,
        route: &ResolvedProxyRoute,
        request_id: &str,
    ) -> Result<Body> {
        let mut upstream_stream = upstream.bytes_stream();
        let model_alias = route.model.alias.clone();
        let request_id = request_id.to_string();

        let stream = stream! {
            let mut buffer = String::new();
            let mut stream_state = AnthropicStreamState::new(request_id, model_alias);

            while let Some(chunk) = upstream_stream.next().await {
                let chunk = match chunk {
                    Ok(chunk) => chunk,
                    Err(error) => {
                        yield Err(io::Error::other(error));
                        return;
                    }
                };
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(event_end) = buffer.find("\n\n") {
                    let raw_event = buffer[..event_end].to_string();
                    buffer.drain(..event_end + 2);

                    if raw_event.trim().is_empty() {
                        continue;
                    }

                    let translated_events = match stream_state.handle_event(&raw_event) {
                        Ok(events) => events,
                        Err(error) => {
                            yield Err(io::Error::other(error));
                            return;
                        }
                    };

                    for translated in translated_events {
                        yield Ok::<Bytes, io::Error>(Bytes::from(translated));
                    }
                }
            }

            for translated in stream_state.finish() {
                yield Ok::<Bytes, io::Error>(Bytes::from(translated));
            }
        };

        Ok(Body::from_stream(stream))
    }

    fn extract_usage(&self, body: &Value) -> UsageSummary {
        UsageSummary {
            prompt_tokens: body
                .get("usage")
                .and_then(|usage| usage.get("input_tokens"))
                .and_then(Value::as_i64),
            completion_tokens: body
                .get("usage")
                .and_then(|usage| usage.get("output_tokens"))
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

struct AnthropicStreamState {
    completion_id: String,
    model_alias: String,
    created: i64,
    sent_role: bool,
    finish_reason: Option<String>,
    done_sent: bool,
}

impl AnthropicStreamState {
    fn new(completion_id: String, model_alias: String) -> Self {
        Self {
            completion_id,
            model_alias,
            created: chrono::Utc::now().timestamp(),
            sent_role: false,
            finish_reason: None,
            done_sent: false,
        }
    }

    fn handle_event(&mut self, raw_event: &str) -> Result<Vec<String>> {
        let mut event_name = "";
        let mut data_lines = Vec::new();

        for line in raw_event.lines() {
            if let Some(rest) = line.strip_prefix("event: ") {
                event_name = rest.trim();
            } else if let Some(rest) = line.strip_prefix("data: ") {
                data_lines.push(rest);
            }
        }

        if data_lines.is_empty() {
            return Ok(Vec::new());
        }

        let payload_text = data_lines.join("\n");
        if payload_text == "[DONE]" {
            return Ok(self.finish());
        }

        let payload: Value = serde_json::from_str(&payload_text)?;
        let mut chunks = Vec::new();

        match event_name {
            "message_start" => {
                if let Some(id) = payload
                    .get("message")
                    .and_then(|message| message.get("id"))
                    .and_then(Value::as_str)
                {
                    self.completion_id = id.to_string();
                }
                if !self.sent_role {
                    chunks.push(self.chunk(&json!({
                        "choices": [{
                            "index": 0,
                            "delta": { "role": "assistant" },
                            "finish_reason": Value::Null,
                        }]
                    })));
                    self.sent_role = true;
                }
            }
            "content_block_delta" => {
                let delta = payload.get("delta").cloned().unwrap_or(Value::Null);
                let text = delta
                    .get("text")
                    .and_then(Value::as_str)
                    .or_else(|| delta.get("thinking").and_then(Value::as_str))
                    .unwrap_or_default();

                if !text.is_empty() {
                    if !self.sent_role {
                        chunks.push(self.chunk(&json!({
                            "choices": [{
                                "index": 0,
                                "delta": { "role": "assistant" },
                                "finish_reason": Value::Null,
                            }]
                        })));
                        self.sent_role = true;
                    }

                    chunks.push(self.chunk(&json!({
                        "choices": [{
                            "index": 0,
                            "delta": { "content": text },
                            "finish_reason": Value::Null,
                        }]
                    })));
                }
            }
            "message_delta" => {
                self.finish_reason = payload
                    .get("delta")
                    .and_then(|delta| delta.get("stop_reason"))
                    .and_then(Value::as_str)
                    .map(map_finish_reason_string);
            }
            "message_stop" => {
                chunks.extend(self.finish());
            }
            "error" => {
                let message = payload
                    .get("error")
                    .and_then(|error| error.get("message"))
                    .and_then(Value::as_str)
                    .unwrap_or("Anthropic streaming error");
                chunks.push(format!(
                    "data: {}\n\n",
                    json!({
                        "error": { "message": message }
                    })
                ));
                chunks.extend(self.finish());
            }
            _ => {}
        }

        Ok(chunks)
    }

    fn finish(&mut self) -> Vec<String> {
        if self.done_sent {
            return Vec::new();
        }

        self.done_sent = true;
        vec![
            self.chunk(&json!({
                "choices": [{
                    "index": 0,
                    "delta": {},
                    "finish_reason": self.finish_reason.clone().unwrap_or_else(|| "stop".to_string()),
                }]
            })),
            "data: [DONE]\n\n".to_string(),
        ]
    }

    fn chunk(&self, body: &Value) -> String {
        let mut chunk = json!({
            "id": self.completion_id,
            "object": "chat.completion.chunk",
            "created": self.created,
            "model": self.model_alias,
        });

        if let Some(map) = chunk.as_object_mut()
            && let Some(body_map) = body.as_object()
        {
            for (key, value) in body_map {
                map.insert(key.clone(), value.clone());
            }
        }

        format!("data: {chunk}\n\n")
    }
}

fn validate_provider_options(
    provider_options: Option<&serde_json::Map<String, Value>>,
) -> Result<()> {
    let Some(provider_options) = provider_options else {
        return Ok(());
    };

    for (key, value) in provider_options {
        if ANTHROPIC_RESERVED_PROVIDER_KEYS.contains(&key.as_str()) {
            bail!("providerOptions.anthropic cannot override `{key}`");
        }

        if !ANTHROPIC_ALLOWED_PROVIDER_KEYS.contains(&key.as_str()) {
            bail!("unsupported anthropic provider option `{key}`");
        }

        if key != "stop_sequences" && value.is_array() {
            bail!("unsupported anthropic provider option `{key}`");
        }

        if !matches!(key.as_str(), "metadata" | "thinking") && value.is_object() {
            bail!("unsupported nested anthropic provider option `{key}`");
        }
    }

    Ok(())
}

fn merge_provider_options(
    body: &mut serde_json::Map<String, Value>,
    provider_options: Option<&serde_json::Map<String, Value>>,
) {
    if let Some(provider_options) = provider_options {
        for (key, value) in provider_options {
            if ANTHROPIC_ALLOWED_PROVIDER_KEYS.contains(&key.as_str()) {
                body.insert(key.clone(), value.clone());
            }
        }
    }
}

fn map_finish_reason(value: Option<&Value>) -> Value {
    Value::String(match value.and_then(Value::as_str) {
        Some(stop_reason) => map_finish_reason_string(stop_reason),
        None => "stop".to_string(),
    })
}

fn map_finish_reason_string(value: &str) -> String {
    match value {
        "end_turn" => "stop".to_string(),
        "max_tokens" => "length".to_string(),
        other => other.to_string(),
    }
}
