use anyhow::{Result, anyhow};
use serde_json::{Map, Value};

#[derive(Clone, Debug)]
pub struct CanonicalChatRequest {
    pub model: String,
    pub messages: Value,
    pub stream: bool,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<i64>,
    pub max_completion_tokens: Option<i64>,
    pub stop: Option<Value>,
    pub presence_penalty: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub n: Option<i64>,
    pub tools: Option<Value>,
    pub response_format: Option<Value>,
    pub provider_options: Map<String, Value>,
    pub raw_body: Value,
}

impl CanonicalChatRequest {
    /// # Errors
    /// Returns an error when required request fields are missing.
    pub fn from_value(mut payload: Value) -> Result<Self> {
        let model = payload
            .get("model")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("`model` is required"))?
            .to_string();

        let provider_options = payload
            .get_mut("providerOptions")
            .and_then(Value::take_object)
            .unwrap_or_default();

        let max_tokens = read_optional_non_negative_i64(&payload, "max_tokens")?;
        let max_completion_tokens =
            read_optional_non_negative_i64(&payload, "max_completion_tokens")?;

        let messages = payload
            .get("messages")
            .cloned()
            .ok_or_else(|| anyhow!("`messages` is required"))?;
        if matches!(&messages, Value::Array(arr) if arr.is_empty()) {
            anyhow::bail!("`messages` must contain at least one message");
        }

        Ok(Self {
            model,
            messages,
            stream: payload
                .get("stream")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            temperature: payload.get("temperature").and_then(Value::as_f64),
            top_p: payload.get("top_p").and_then(Value::as_f64),
            max_tokens,
            max_completion_tokens,
            stop: payload.get("stop").cloned(),
            presence_penalty: payload.get("presence_penalty").and_then(Value::as_f64),
            frequency_penalty: payload.get("frequency_penalty").and_then(Value::as_f64),
            n: payload.get("n").and_then(Value::as_i64),
            tools: payload.get("tools").cloned(),
            response_format: payload.get("response_format").cloned(),
            provider_options,
            raw_body: payload,
        })
    }

    #[must_use]
    pub fn provider_options_for(&self, provider: &str) -> Option<&Map<String, Value>> {
        self.provider_options.get(provider)?.as_object()
    }
}

trait TakeObject {
    fn take_object(&mut self) -> Option<Map<String, Value>>;
}

impl TakeObject for Value {
    fn take_object(&mut self) -> Option<Map<String, Value>> {
        match std::mem::take(self) {
            Value::Object(map) => Some(map),
            _ => None,
        }
    }
}

fn read_optional_non_negative_i64(payload: &Value, field: &str) -> Result<Option<i64>> {
    let Some(value) = payload.get(field) else {
        return Ok(None);
    };

    let parsed = value
        .as_i64()
        .ok_or_else(|| anyhow!("`{field}` must be an integer"))?;

    if parsed < 0 {
        return Err(anyhow!("`{field}` must be non-negative"));
    }

    Ok(Some(parsed))
}

#[cfg(test)]
mod tests {
    use super::CanonicalChatRequest;
    use serde_json::json;

    #[test]
    fn rejects_negative_max_tokens() {
        let payload = json!({
            "model": "google-genai/gemini-2.5-flash",
            "messages": [{"role":"user","content":"hi"}],
            "max_tokens": -5
        });

        let error = CanonicalChatRequest::from_value(payload).unwrap_err();
        assert_eq!(error.to_string(), "`max_tokens` must be non-negative");
    }

    #[test]
    fn rejects_negative_max_completion_tokens() {
        let payload = json!({
            "model": "google-genai/gemini-2.5-flash",
            "messages": [{"role":"user","content":"hi"}],
            "max_completion_tokens": -10
        });

        let error = CanonicalChatRequest::from_value(payload).unwrap_err();
        assert_eq!(
            error.to_string(),
            "`max_completion_tokens` must be non-negative"
        );
    }

    #[test]
    fn rejects_empty_messages() {
        let payload = json!({
            "model": "google-genai/gemini-2.5-flash",
            "messages": []
        });

        let error = CanonicalChatRequest::from_value(payload).unwrap_err();
        assert_eq!(
            error.to_string(),
            "`messages` must contain at least one message"
        );
    }

    #[test]
    fn rejects_non_integer_max_tokens() {
        let payload = json!({
            "model": "google-genai/gemini-2.5-flash",
            "messages": [{"role":"user","content":"hi"}],
            "max_tokens": "not-a-number"
        });

        let error = CanonicalChatRequest::from_value(payload).unwrap_err();
        assert_eq!(error.to_string(), "`max_tokens` must be an integer");
    }
}
