use anyhow::{Result, anyhow, bail};
use async_stream::stream;
use axum::{body::Body, http::HeaderMap};
use bytes::Bytes;
use futures_util::StreamExt;
use serde_json::{Value, json};
use std::io;

use super::{CanonicalChatRequest, ProviderAdapter, UsageSummary, openai_stream_headers};
use valymux_surrealdb::ResolvedProxyRoute;

pub struct GoogleGenAiAdapter;

const GENAI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

impl ProviderAdapter for GoogleGenAiAdapter {
    fn name(&self) -> &'static str {
        "google-genai"
    }

    fn target_url(&self) -> &'static str {
        // Not used directly — request_url() below builds the full dynamic URL.
        GENAI_BASE_URL
    }

    fn request_url(&self, route: &ResolvedProxyRoute, stream: bool) -> String {
        let model = &route.model.upstream_model;
        let endpoint = if stream {
            "streamGenerateContent?alt=sse"
        } else {
            "generateContent"
        };
        format!("{GENAI_BASE_URL}/{model}:{endpoint}")
    }

    fn validate_request(
        &self,
        _request: &CanonicalChatRequest,
        _route: &ResolvedProxyRoute,
    ) -> Result<()> {
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

        let mut system_parts: Vec<String> = Vec::new();
        let mut contents: Vec<Value> = Vec::new();

        for message in messages {
            let role = message
                .get("role")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("message role is required"))?;
            let content = message
                .get("content")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    anyhow!("only string message content is supported for google-genai in MVP")
                })?;

            match role {
                "system" => system_parts.push(content.to_string()),
                "user" => contents.push(json!({
                    "role": "user",
                    "parts": [{ "text": content }],
                })),
                "assistant" => contents.push(json!({
                    "role": "model",
                    "parts": [{ "text": content }],
                })),
                _ => bail!("message role `{role}` is not supported for google-genai in MVP"),
            }
        }

        let max_output_tokens = request
            .max_completion_tokens
            .or(request.max_tokens)
            .unwrap_or(route.model.max_output_tokens)
            .min(route.model.max_output_tokens);

        let mut generation_config = serde_json::Map::new();
        generation_config.insert("maxOutputTokens".to_string(), json!(max_output_tokens));

        if let Some(temperature) = request.temperature {
            generation_config.insert("temperature".to_string(), json!(temperature));
        }

        if let Some(top_p) = request.top_p {
            generation_config.insert("topP".to_string(), json!(top_p));
        }

        if let Some(stop) = &request.stop {
            if let Some(arr) = stop.as_array() {
                generation_config.insert("stopSequences".to_string(), json!(arr));
            } else if let Some(s) = stop.as_str() {
                generation_config.insert("stopSequences".to_string(), json!([s]));
            }
        }

        let mut body = json!({
            "contents": contents,
            "generationConfig": generation_config,
        });

        if !system_parts.is_empty() {
            body["systemInstruction"] = json!({
                "parts": [{ "text": system_parts.join("\n\n") }],
            });
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
            .header("x-goog-api-key", api_key)
            .header("x-request-id", request_id)
    }

    fn translate_json_response(
        &self,
        body: &Value,
        _request: &CanonicalChatRequest,
        route: &ResolvedProxyRoute,
    ) -> Value {
        let candidate = body
            .get("candidates")
            .and_then(Value::as_array)
            .and_then(|arr| arr.first());

        let content = candidate
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(Value::as_array)
            .map(|parts| {
                parts
                    .iter()
                    .filter_map(|p| p.get("text").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();

        let finish_reason = candidate
            .and_then(|c| c.get("finishReason"))
            .and_then(Value::as_str)
            .map(map_finish_reason_string)
            .unwrap_or_else(|| "stop".to_string());

        let usage = body.get("usageMetadata");
        let prompt_tokens = usage
            .and_then(|u| u.get("promptTokenCount"))
            .and_then(Value::as_i64);
        let completion_tokens = usage
            .and_then(|u| u.get("candidatesTokenCount"))
            .and_then(Value::as_i64);

        json!({
            "id": format!("chatcmpl-{}", uuid::Uuid::new_v4().simple()),
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
                "finish_reason": finish_reason,
            }],
            "usage": {
                "prompt_tokens": prompt_tokens,
                "completion_tokens": completion_tokens,
                "total_tokens": match (prompt_tokens, completion_tokens) {
                    (Some(p), Some(c)) => Some(p + c),
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
        let completion_id = request_id.to_string();

        let stream = stream! {
            let mut buffer = String::new();
            let mut state = GenAiStreamState::new(completion_id, model_alias);

            while let Some(chunk) = upstream_stream.next().await {
                let chunk = match chunk {
                    Ok(chunk) => chunk,
                    Err(error) => {
                        yield Err(io::Error::other(error));
                        return;
                    }
                };
                // Normalize CRLF → LF so event splitting works regardless of line endings
                buffer.push_str(&String::from_utf8_lossy(&chunk).replace("\r\n", "\n"));

                // Gemini SSE: events separated by \n\n (after CRLF normalisation)
                while let Some(event_end) = buffer.find("\n\n") {
                    let raw_event = buffer[..event_end].to_string();
                    buffer.drain(..event_end + 2);

                    if raw_event.trim().is_empty() {
                        continue;
                    }

                    let translated_events = match state.handle_event(&raw_event) {
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

            for translated in state.finish() {
                yield Ok::<Bytes, io::Error>(Bytes::from(translated));
            }
        };

        Ok(Body::from_stream(stream))
    }

    fn extract_usage(&self, body: &Value) -> UsageSummary {
        let usage = body.get("usageMetadata");
        UsageSummary {
            prompt_tokens: usage
                .and_then(|u| u.get("promptTokenCount"))
                .and_then(Value::as_i64),
            completion_tokens: usage
                .and_then(|u| u.get("candidatesTokenCount"))
                .and_then(Value::as_i64),
        }
    }

    fn error_message(&self, body: &Value) -> Option<String> {
        body.get("error")
            .and_then(|e| e.get("message"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
    }

    fn stream_headers(&self) -> Option<HeaderMap> {
        Some(openai_stream_headers())
    }
}

// ---------------------------------------------------------------------------
// Streaming state machine
// ---------------------------------------------------------------------------

struct GenAiStreamState {
    completion_id: String,
    model_alias: String,
    created: i64,
    sent_role: bool,
    finish_reason: Option<String>,
    done_sent: bool,
}

impl GenAiStreamState {
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
        // Extract data lines (Gemini SSE has no separate `event:` field)
        let data_text: String = raw_event
            .lines()
            .filter_map(|line| line.strip_prefix("data: "))
            .collect::<Vec<_>>()
            .join("\n");

        if data_text.is_empty() {
            return Ok(Vec::new());
        }

        if data_text.trim() == "[DONE]" {
            return Ok(self.finish());
        }

        let payload: Value = serde_json::from_str(&data_text)?;
        let mut chunks = Vec::new();

        // Emit role delta on first content
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

        // Extract text from candidates[0].content.parts
        let text = payload
            .get("candidates")
            .and_then(Value::as_array)
            .and_then(|arr| arr.first())
            .and_then(|c| {
                // Capture finish reason while we have the candidate
                if let Some(reason) = c.get("finishReason").and_then(Value::as_str) {
                    self.finish_reason = Some(map_finish_reason_string(reason));
                }
                c.get("content")
            })
            .and_then(|c| c.get("parts"))
            .and_then(Value::as_array)
            .map(|parts| {
                parts
                    .iter()
                    .filter_map(|p| p.get("text").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();

        if !text.is_empty() {
            chunks.push(self.chunk(&json!({
                "choices": [{
                    "index": 0,
                    "delta": { "content": text },
                    "finish_reason": Value::Null,
                }]
            })));
        }

        // If this chunk signals the end, flush
        if self.finish_reason.is_some() {
            chunks.extend(self.finish());
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn map_finish_reason_string(value: &str) -> String {
    match value {
        "STOP" => "stop".to_string(),
        "MAX_TOKENS" => "length".to_string(),
        "SAFETY" => "content_filter".to_string(),
        other => other.to_lowercase(),
    }
}
