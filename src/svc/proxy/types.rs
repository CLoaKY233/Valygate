use anyhow::{Result, anyhow};
use serde_json::{Map, Value};

#[derive(Clone, Debug)]
pub struct CanonicalChatRequest {
    pub model: String,
    pub stream: bool,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<i64>,
    pub max_completion_tokens: Option<i64>,
    pub tools: Option<Value>,
    pub provider_options: Map<String, Value>,
    pub raw_body: Value,
}

impl CanonicalChatRequest {
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

        Ok(Self {
            model,
            stream: payload
                .get("stream")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            temperature: payload.get("temperature").and_then(Value::as_f64),
            top_p: payload.get("top_p").and_then(Value::as_f64),
            max_tokens: payload.get("max_tokens").and_then(Value::as_i64),
            max_completion_tokens: payload.get("max_completion_tokens").and_then(Value::as_i64),
            tools: payload.get("tools").cloned(),
            provider_options,
            raw_body: payload,
        })
    }

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
