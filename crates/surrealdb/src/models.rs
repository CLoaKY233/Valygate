use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Thing,
    pub name: String,
    pub email: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthSession {
    pub user: User,
    pub token: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProviderKind {
    #[serde(rename = "openai")]
    OpenAi,
    #[serde(rename = "anthropic")]
    Anthropic,
}

impl ProviderKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderCredential {
    pub id: Thing,
    pub user: Thing,
    pub provider: String,
    pub label: String,
    pub encrypted_api_key: String,
    pub tags: Vec<String>,
    pub enabled: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VirtualApiKey {
    pub id: Thing,
    pub user: Thing,
    pub name: String,
    pub key_prefix: String,
    pub key_hash: String,
    pub allowed_models: Vec<String>,
    pub tags: Vec<String>,
    pub enabled: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelCatalogEntry {
    pub id: Thing,
    pub alias: String,
    pub display_name: String,
    pub provider: String,
    pub upstream_model: String,
    pub description: String,
    pub tags: Vec<String>,
    pub enabled: bool,
    pub context_window_tokens: i64,
    pub max_output_tokens: i64,
    pub supports_streaming: bool,
    pub supports_thinking: bool,
    pub thinking_required: bool,
    pub supports_temperature: bool,
    pub temperature_fixed_to: Option<f64>,
    pub temperature_min: Option<f64>,
    pub temperature_max: Option<f64>,
    pub supports_top_p: bool,
    pub supports_system_messages: bool,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_json_mode: bool,
    pub supports_parallel_tool_calls: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestLog {
    pub id: Thing,
    pub request_id: String,
    pub user: Thing,
    pub virtual_api_key: Option<Thing>,
    pub model_alias: String,
    pub provider: String,
    pub upstream_model: String,
    pub status_code: i64,
    pub latency_ms: i64,
    pub stream: bool,
    pub request_url: String,
    pub error_message: Option<String>,
    pub usage_input_tokens: Option<i64>,
    pub usage_output_tokens: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignupInput {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SigninInput {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateProfileInput {
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateProviderCredentialInput {
    pub provider: ProviderKind,
    pub label: String,
    pub api_key: String,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateProviderCredentialInput {
    pub label: String,
    pub api_key: Option<String>,
    pub tags: Vec<String>,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateVirtualApiKeyInput {
    pub name: String,
    pub allowed_models: Vec<String>,
    pub tags: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateVirtualApiKeyInput {
    pub name: String,
    pub allowed_models: Vec<String>,
    pub tags: Vec<String>,
    pub enabled: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatedVirtualApiKey {
    pub record: VirtualApiKey,
    pub raw_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifiedVirtualApiKey {
    pub key: VirtualApiKey,
    pub user_id: Thing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResolvedProxyRoute {
    pub user: User,
    pub key: VirtualApiKey,
    pub model: ModelCatalogEntry,
    pub provider_credential: ProviderCredential,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestLogInput {
    pub request_id: String,
    pub user_id: String,
    pub virtual_api_key_id: Option<String>,
    pub model_alias: String,
    pub provider: String,
    pub upstream_model: String,
    pub status_code: i64,
    pub latency_ms: i64,
    pub stream: bool,
    pub request_url: String,
    pub error_message: Option<String>,
    pub usage_input_tokens: Option<i64>,
    pub usage_output_tokens: Option<i64>,
}
