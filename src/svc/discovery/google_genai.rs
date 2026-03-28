use anyhow::{Result, anyhow, bail};
use serde_json::Value;
use valymux_surrealdb::ModelSyncInput;

const LIST_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/models?pageSize=1000";

pub async fn discover_models(
    api_key: &str,
    client: &reqwest::Client,
) -> Result<Vec<ModelSyncInput>> {
    let url = format!("{LIST_URL}&key={api_key}");
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        bail!("Google GenAI API returned {status}: {body}");
    }

    let data: Value = response.json().await?;

    let models = data["models"]
        .as_array()
        .ok_or_else(|| anyhow!("unexpected response: no `models` array"))?;

    models
        .iter()
        .filter(|m| supports_generate_content(m))
        .map(map_model)
        .collect()
}

fn supports_generate_content(model: &Value) -> bool {
    model["supportedGenerationMethods"]
        .as_array()
        .map(|methods| {
            methods
                .iter()
                .any(|m| m.as_str() == Some("generateContent"))
        })
        .unwrap_or(false)
}

fn map_model(m: &Value) -> Result<ModelSyncInput> {
    // "models/gemini-2.5-flash" → "gemini-2.5-flash"
    let full_name = m["name"]
        .as_str()
        .ok_or_else(|| anyhow!("model has no name field"))?;

    let upstream_model = full_name
        .strip_prefix("models/")
        .unwrap_or(full_name)
        .to_string();

    let alias = format!("google-genai/{upstream_model}");

    let display_name = m["displayName"]
        .as_str()
        .unwrap_or(&upstream_model)
        .to_string();

    let description = m["description"].as_str().map(ToOwned::to_owned);

    let context_window_tokens = m["inputTokenLimit"].as_i64().unwrap_or(0);
    let max_output_tokens = m["outputTokenLimit"].as_i64().unwrap_or(0);

    // All Google GenAI generateContent models support streamGenerateContent
    // even though the API doesn't advertise it in supportedGenerationMethods.
    let supports_streaming = true;

    let supports_thinking = m["thinking"].as_bool().unwrap_or(false);

    let temperature_max = m["maxTemperature"].as_f64();
    let top_p_default = m["topP"].as_f64();

    // Gemini models support temperature when maxTemperature is present
    let supports_temperature = temperature_max.is_some();
    // Gemini models support top_p when topP is present
    let supports_top_p = top_p_default.is_some();

    Ok(ModelSyncInput {
        alias,
        provider: "google-genai".to_string(),
        upstream_model,
        display_name,
        description,
        context_window_tokens,
        max_output_tokens,
        supports_streaming,
        supports_thinking,
        thinking_required: false,
        supports_temperature,
        temperature_fixed_to: None,
        temperature_min: if supports_temperature { Some(0.0) } else { None },
        temperature_max,
        supports_top_p,
        supports_system_messages: true,
        supports_tools: true,
        supports_vision: true,
        supports_json_mode: true,
        supports_parallel_tool_calls: true,
    })
}
