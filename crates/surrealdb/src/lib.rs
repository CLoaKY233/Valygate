//! `SurrealDB` facade for `ValyGate`.
//!
//! This crate currently keeps one long-lived root client for bootstrap, catalog reads, and
//! request logging. User-scoped operations still create per-request authenticated clients so
//! record-access permissions are enforced correctly. That avoids accidental auth-state sharing,
//! but it also adds connection setup overhead on user-heavy paths. If authenticated control-plane
//! traffic becomes a throughput bottleneck, this is the point to introduce a token-keyed client
//! pool or another session-reuse strategy.

mod config;
mod crypto;
mod error;
mod models;
mod schema;

use serde::Serialize;
use surrealdb::{
    Surreal,
    engine::any,
    opt::auth::{Record, Root, Token},
    types::{RecordId, SurrealValue, ToSql},
};
use tracing::info;

pub use config::DatabaseConfig;
use crypto::{
    decrypt_secret, encrypt_secret, generate_virtual_api_key, hash_virtual_api_key, key_prefix,
};
pub use error::DatabaseError;
pub use models::{
    AuthSession, CreateProviderCredentialInput, CreateVirtualApiKeyInput, CreatedVirtualApiKey,
    ModelCatalogEntry, ProviderCredential, ProviderKind, RequestLog, RequestLogInput,
    ResolvedProxyRoute, SigninInput, SignupInput, UpdateProfileInput,
    UpdateProviderCredentialInput, UpdateVirtualApiKeyInput, User, VerifiedVirtualApiKey,
    VirtualApiKey,
};
use schema::{SCHEMA_FILES, validate_schema_files};

#[derive(Clone)]
pub struct Database {
    client: Surreal<any::Any>,
    config: DatabaseConfig,
    encryption_key: [u8; 32],
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Serialize, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct SeedModel {
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

fn openai_gpt_4o_mini_seed() -> (&'static str, SeedModel) {
    (
        "model_catalog:openai_gpt_4o_mini",
        SeedModel {
            alias: "gpt-4o-mini".to_string(),
            display_name: "GPT-4o Mini".to_string(),
            provider: "openai".to_string(),
            upstream_model: "gpt-4o-mini".to_string(),
            description: "Fast low-cost OpenAI chat model.".to_string(),
            tags: vec!["chat".to_string(), "fast".to_string()],
            enabled: true,
            context_window_tokens: 128_000,
            max_output_tokens: 16_384,
            supports_streaming: true,
            supports_thinking: false,
            thinking_required: false,
            supports_temperature: true,
            temperature_fixed_to: None,
            temperature_min: Some(0.0),
            temperature_max: Some(2.0),
            supports_top_p: true,
            supports_system_messages: true,
            supports_tools: true,
            supports_vision: true,
            supports_json_mode: true,
            supports_parallel_tool_calls: true,
        },
    )
}

fn openai_gpt_4o_seed() -> (&'static str, SeedModel) {
    (
        "model_catalog:openai_gpt_4o",
        SeedModel {
            alias: "gpt-4o".to_string(),
            display_name: "GPT-4o".to_string(),
            provider: "openai".to_string(),
            upstream_model: "gpt-4o".to_string(),
            description: "General-purpose OpenAI flagship model.".to_string(),
            tags: vec!["chat".to_string(), "flagship".to_string()],
            enabled: true,
            context_window_tokens: 128_000,
            max_output_tokens: 16_384,
            supports_streaming: true,
            supports_thinking: false,
            thinking_required: false,
            supports_temperature: true,
            temperature_fixed_to: None,
            temperature_min: Some(0.0),
            temperature_max: Some(2.0),
            supports_top_p: true,
            supports_system_messages: true,
            supports_tools: true,
            supports_vision: true,
            supports_json_mode: true,
            supports_parallel_tool_calls: true,
        },
    )
}

fn anthropic_claude_3_7_sonnet_seed() -> (&'static str, SeedModel) {
    (
        "model_catalog:anthropic_claude_3_7_sonnet",
        SeedModel {
            alias: "claude-3-7-sonnet".to_string(),
            display_name: "Claude 3.7 Sonnet".to_string(),
            provider: "anthropic".to_string(),
            upstream_model: "claude-3-7-sonnet-latest".to_string(),
            description: "Anthropic reasoning-capable chat model.".to_string(),
            tags: vec!["chat".to_string(), "thinking".to_string()],
            enabled: true,
            context_window_tokens: 200_000,
            max_output_tokens: 8_192,
            supports_streaming: true,
            supports_thinking: true,
            thinking_required: false,
            supports_temperature: true,
            temperature_fixed_to: Some(1.0),
            temperature_min: Some(1.0),
            temperature_max: Some(1.0),
            supports_top_p: false,
            supports_system_messages: true,
            supports_tools: false,
            supports_vision: true,
            supports_json_mode: false,
            supports_parallel_tool_calls: false,
        },
    )
}

fn model_catalog_seeds() -> [(&'static str, SeedModel); 3] {
    [
        openai_gpt_4o_mini_seed(),
        openai_gpt_4o_seed(),
        anthropic_claude_3_7_sonnet_seed(),
    ]
}

#[allow(clippy::missing_errors_doc)]
impl Database {
    pub async fn connect(config: DatabaseConfig) -> Result<Self, DatabaseError> {
        config.validate()?;
        let encryption_key = config.encryption_key_bytes()?;

        let client = any::connect(&config.surreal_url).await?;
        client
            .use_ns(&config.surreal_namespace)
            .use_db(&config.surreal_database)
            .await?;
        client
            .signin(Root {
                username: config.surreal_username.clone(),
                password: config.surreal_password.clone(),
            })
            .await?;

        Ok(Self {
            client,
            config,
            encryption_key,
        })
    }

    pub async fn bootstrap(&self) -> Result<(), DatabaseError> {
        validate_schema_files()?;

        for schema_file in SCHEMA_FILES {
            info!(path = schema_file.path, "Applying SurrealDB schema file");
            self.client.query(schema_file.contents).await?;
        }

        self.seed_model_catalog().await?;
        Ok(())
    }

    #[must_use]
    pub fn client(&self) -> &Surreal<any::Any> {
        &self.client
    }

    pub async fn signup_user(&self, input: SignupInput) -> Result<AuthSession, DatabaseError> {
        #[derive(Serialize, SurrealValue)]
        #[surreal(crate = "surrealdb::types")]
        struct Params {
            name: String,
            email: String,
            password: String,
        }

        let user_client = self.fresh_client().await?;
        let token: Token = user_client
            .signup(Record {
                access: "account".to_string(),
                namespace: self.config.surreal_namespace.clone(),
                database: self.config.surreal_database.clone(),
                params: Params {
                    name: input.name.clone(),
                    email: input.email.clone(),
                    password: input.password.clone(),
                },
            })
            .await?;

        let user = user_client
            .query("SELECT * FROM $auth.id;")
            .await?
            .take::<Option<User>>(0)?
            .ok_or_else(|| DatabaseError::SchemaBootstrap("signed up user was not found".into()))?;

        Ok(AuthSession {
            user,
            token: token.access.into_insecure_token(),
        })
    }

    pub async fn signin_user(&self, input: SigninInput) -> Result<AuthSession, DatabaseError> {
        #[derive(Serialize, SurrealValue)]
        #[surreal(crate = "surrealdb::types")]
        struct Params {
            email: String,
            password: String,
        }

        let user_client = self.fresh_client().await?;
        let token: Token = user_client
            .signin(Record {
                access: "account".to_string(),
                namespace: self.config.surreal_namespace.clone(),
                database: self.config.surreal_database.clone(),
                params: Params {
                    email: input.email.clone(),
                    password: input.password.clone(),
                },
            })
            .await?;

        let user = user_client
            .query("SELECT * FROM $auth.id;")
            .await?
            .take::<Option<User>>(0)?
            .ok_or_else(|| DatabaseError::SchemaBootstrap("signed in user was not found".into()))?;

        Ok(AuthSession {
            user,
            token: token.access.into_insecure_token(),
        })
    }

    pub async fn authenticate_user(&self, token: &str) -> Result<User, DatabaseError> {
        let client = self.fresh_client_with_token(token).await?;
        client
            .query("SELECT * FROM $auth.id;")
            .await?
            .take::<Option<User>>(0)?
            .ok_or_else(|| DatabaseError::InvalidConfig("authenticated user not found".into()))
    }

    pub async fn update_profile(
        &self,
        token: &str,
        input: UpdateProfileInput,
    ) -> Result<User, DatabaseError> {
        let client = self.fresh_client_with_token(token).await?;
        client
            .query("UPDATE $auth.id MERGE { name: $name, updated_at: time::now() };")
            .bind(("name", input.name))
            .await?;
        self.authenticate_user(token).await
    }

    pub async fn list_provider_credentials(
        &self,
        token: &str,
    ) -> Result<Vec<ProviderCredential>, DatabaseError> {
        let client = self.fresh_client_with_token(token).await?;
        client
            .query("SELECT * FROM provider_credential ORDER BY created_at ASC;")
            .await?
            .take::<Vec<ProviderCredential>>(0)
            .map_err(Into::into)
    }

    pub async fn get_provider_credential(
        &self,
        token: &str,
        provider_id: &str,
    ) -> Result<Option<ProviderCredential>, DatabaseError> {
        let client = self.fresh_client_with_token(token).await?;
        let provider_id = parse_thing(provider_id)?;
        client
            .query("SELECT * FROM $provider_id;")
            .bind(("provider_id", provider_id))
            .await?
            .take::<Option<ProviderCredential>>(0)
            .map_err(Into::into)
    }

    pub async fn create_provider_credential(
        &self,
        token: &str,
        input: CreateProviderCredentialInput,
    ) -> Result<ProviderCredential, DatabaseError> {
        let client = self.fresh_client_with_token(token).await?;
        let encrypted_api_key = encrypt_secret(&self.encryption_key, &input.api_key)?;

        let record = client
            .query(
                "CREATE provider_credential CONTENT {
                    user: $auth.id,
                    provider: $provider,
                    label: $label,
                    encrypted_api_key: $encrypted_api_key,
                    tags: $tags,
                    enabled: true
                };",
            )
            .bind(("provider", input.provider.as_str().to_string()))
            .bind(("label", input.label))
            .bind(("encrypted_api_key", encrypted_api_key))
            .bind(("tags", input.tags))
            .await?
            .take::<Option<ProviderCredential>>(0)?
            .ok_or_else(|| {
                DatabaseError::SchemaBootstrap("provider creation returned no record".into())
            })?;

        Ok(record)
    }

    pub async fn update_provider_credential(
        &self,
        token: &str,
        provider_id: &str,
        input: UpdateProviderCredentialInput,
    ) -> Result<Option<ProviderCredential>, DatabaseError> {
        let client = self.fresh_client_with_token(token).await?;
        let provider_id = parse_thing(provider_id)?;
        let encrypted_api_key = match input.api_key {
            Some(api_key) => Some(encrypt_secret(&self.encryption_key, &api_key)?),
            None => None,
        };

        client
            .query(
                "LET $update = {
                    label: $label,
                    tags: $tags,
                    enabled: $enabled,
                    updated_at: time::now()
                };
                UPDATE $provider_id MERGE (
                    $encrypted_api_key = NONE
                        ? $update
                        : $update.merge({ encrypted_api_key: $encrypted_api_key })
                );",
            )
            .bind(("provider_id", provider_id.clone()))
            .bind(("label", input.label))
            .bind(("tags", input.tags))
            .bind(("enabled", input.enabled))
            .bind(("encrypted_api_key", encrypted_api_key))
            .await?;

        self.get_provider_credential(token, &provider_id.to_sql())
            .await
    }

    pub async fn delete_provider_credential(
        &self,
        token: &str,
        provider_id: &str,
    ) -> Result<Option<ProviderCredential>, DatabaseError> {
        let client = self.fresh_client_with_token(token).await?;
        let provider_id = parse_thing(provider_id)?;
        client
            .query("DELETE $provider_id RETURN BEFORE;")
            .bind(("provider_id", provider_id))
            .await?
            .take::<Option<ProviderCredential>>(0)
            .map_err(Into::into)
    }

    pub async fn list_virtual_api_keys(
        &self,
        token: &str,
    ) -> Result<Vec<VirtualApiKey>, DatabaseError> {
        let client = self.fresh_client_with_token(token).await?;
        client
            .query("SELECT * FROM virtual_api_key ORDER BY created_at ASC;")
            .await?
            .take::<Vec<VirtualApiKey>>(0)
            .map_err(Into::into)
    }

    pub async fn get_virtual_api_key(
        &self,
        token: &str,
        key_id: &str,
    ) -> Result<Option<VirtualApiKey>, DatabaseError> {
        let client = self.fresh_client_with_token(token).await?;
        let key_id = parse_thing(key_id)?;
        client
            .query("SELECT * FROM $key_id;")
            .bind(("key_id", key_id))
            .await?
            .take::<Option<VirtualApiKey>>(0)
            .map_err(Into::into)
    }

    pub async fn create_virtual_api_key(
        &self,
        token: &str,
        input: CreateVirtualApiKeyInput,
    ) -> Result<CreatedVirtualApiKey, DatabaseError> {
        self.ensure_model_aliases_exist(&input.allowed_models)
            .await?;

        let client = self.fresh_client_with_token(token).await?;
        let raw_key = generate_virtual_api_key()?;

        let created = client
            .query(
                "CREATE virtual_api_key CONTENT {
                    user: $auth.id,
                    name: $name,
                    key_prefix: $key_prefix,
                    key_hash: $key_hash,
                    allowed_models: $allowed_models,
                    tags: $tags,
                    enabled: true,
                    expires_at: $expires_at
                };",
            )
            .bind(("name", input.name))
            .bind(("key_prefix", key_prefix(&raw_key)))
            .bind(("key_hash", hash_virtual_api_key(&raw_key)))
            .bind(("allowed_models", input.allowed_models))
            .bind(("tags", input.tags))
            .bind(("expires_at", input.expires_at))
            .await?
            .take::<Option<VirtualApiKey>>(0)?
            .ok_or_else(|| {
                DatabaseError::SchemaBootstrap("virtual key creation returned no record".into())
            })?;

        Ok(CreatedVirtualApiKey {
            record: created,
            raw_key,
        })
    }

    pub async fn update_virtual_api_key(
        &self,
        token: &str,
        key_id: &str,
        input: UpdateVirtualApiKeyInput,
    ) -> Result<Option<VirtualApiKey>, DatabaseError> {
        self.ensure_model_aliases_exist(&input.allowed_models)
            .await?;

        let client = self.fresh_client_with_token(token).await?;
        let key_id = parse_thing(key_id)?;
        client
            .query(
                "UPDATE $key_id MERGE {
                    name: $name,
                    allowed_models: $allowed_models,
                    tags: $tags,
                    enabled: $enabled,
                    expires_at: $expires_at,
                    updated_at: time::now()
                };",
            )
            .bind(("key_id", key_id.clone()))
            .bind(("name", input.name))
            .bind(("allowed_models", input.allowed_models))
            .bind(("tags", input.tags))
            .bind(("enabled", input.enabled))
            .bind(("expires_at", input.expires_at))
            .await?;

        self.get_virtual_api_key(token, &key_id.to_sql()).await
    }

    pub async fn delete_virtual_api_key(
        &self,
        token: &str,
        key_id: &str,
    ) -> Result<Option<VirtualApiKey>, DatabaseError> {
        let client = self.fresh_client_with_token(token).await?;
        let key_id = parse_thing(key_id)?;
        client
            .query("DELETE $key_id RETURN BEFORE;")
            .bind(("key_id", key_id))
            .await?
            .take::<Option<VirtualApiKey>>(0)
            .map_err(Into::into)
    }

    pub async fn list_usable_models(
        &self,
        token: &str,
    ) -> Result<Vec<ModelCatalogEntry>, DatabaseError> {
        let user = self.authenticate_user(token).await?;
        self.list_usable_models_for_user(&user.id.to_sql()).await
    }

    pub async fn get_model_by_alias_for_user(
        &self,
        token: &str,
        alias: &str,
    ) -> Result<Option<ModelCatalogEntry>, DatabaseError> {
        let user = self.authenticate_user(token).await?;
        let models = self.list_usable_models_for_user(&user.id.to_sql()).await?;
        Ok(models.into_iter().find(|model| model.alias == alias))
    }

    pub async fn list_usable_models_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<ModelCatalogEntry>, DatabaseError> {
        let user_id = parse_thing(user_id)?;
        self.client
            .query(
                "SELECT * FROM model_catalog
                 WHERE enabled = true
                   AND provider IN (
                        SELECT VALUE provider FROM provider_credential
                        WHERE user = $user_id AND enabled = true
                   )
                 ORDER BY alias ASC;",
            )
            .bind(("user_id", user_id))
            .await?
            .take::<Vec<ModelCatalogEntry>>(0)
            .map_err(Into::into)
    }

    pub async fn verify_virtual_api_key(
        &self,
        raw_key: &str,
    ) -> Result<Option<VerifiedVirtualApiKey>, DatabaseError> {
        let key_hash = hash_virtual_api_key(raw_key);
        let key = self
            .client
            .query(
                "SELECT * FROM virtual_api_key
                 WHERE key_hash = $key_hash
                   AND enabled = true
                   AND (expires_at = NONE OR expires_at > time::now())
                 LIMIT 1;",
            )
            .bind(("key_hash", key_hash))
            .await?
            .take::<Option<VirtualApiKey>>(0)?;

        Ok(key.map(|record| VerifiedVirtualApiKey {
            user_id: record.user.clone(),
            key: record,
        }))
    }

    pub async fn resolve_proxy_route(
        &self,
        raw_key: &str,
        requested_model: &str,
    ) -> Result<ResolvedProxyRoute, DatabaseError> {
        let requested_model = requested_model.to_owned();
        let verified = self
            .verify_virtual_api_key(raw_key)
            .await?
            .ok_or_else(|| DatabaseError::InvalidConfig("virtual API key is invalid".into()))?;

        if !verified
            .key
            .allowed_models
            .iter()
            .any(|model| model == &requested_model)
        {
            return Err(DatabaseError::InvalidConfig(
                "virtual API key is not allowed to use this model".into(),
            ));
        }

        let user = self
            .client
            .query("SELECT * FROM $user_id;")
            .bind(("user_id", verified.user_id.clone()))
            .await?
            .take::<Option<User>>(0)?
            .ok_or_else(|| {
                DatabaseError::InvalidConfig("user for virtual key was not found".into())
            })?;

        if !user.enabled {
            return Err(DatabaseError::InvalidConfig(
                "user for virtual key is disabled".into(),
            ));
        }

        let model = self
            .client
            .query(
                "SELECT * FROM model_catalog
                 WHERE alias = $alias
                   AND enabled = true
                 LIMIT 1;",
            )
            .bind(("alias", requested_model))
            .await?
            .take::<Option<ModelCatalogEntry>>(0)?
            .ok_or_else(|| DatabaseError::InvalidConfig("requested model was not found".into()))?;

        let provider_credential = self
            .client
            .query(
                "SELECT * FROM provider_credential
                 WHERE user = $user_id
                   AND provider = $provider
                   AND enabled = true
                 ORDER BY updated_at DESC
                 LIMIT 1;",
            )
            .bind(("user_id", user.id.clone()))
            .bind(("provider", model.provider.clone()))
            .await?
            .take::<Option<ProviderCredential>>(0)?
            .ok_or_else(|| {
                DatabaseError::InvalidConfig(
                    "no enabled provider credential exists for this model".into(),
                )
            })?;

        Ok(ResolvedProxyRoute {
            user,
            key: verified.key,
            model,
            provider_credential,
        })
    }

    pub async fn log_request(
        &self,
        input: RequestLogInput,
    ) -> Result<Option<RequestLog>, DatabaseError> {
        #[derive(Serialize, SurrealValue)]
        #[surreal(crate = "surrealdb::types")]
        struct RequestLogRecord {
            request_id: String,
            user: RecordId,
            virtual_api_key: Option<RecordId>,
            model_alias: String,
            provider: String,
            upstream_model: String,
            status_code: i64,
            latency_ms: i64,
            stream: bool,
            request_url: String,
            error_message: Option<String>,
            usage_input_tokens: Option<i64>,
            usage_output_tokens: Option<i64>,
        }

        let user = parse_thing(&input.user_id)?;
        let virtual_api_key = input
            .virtual_api_key_id
            .as_deref()
            .map(parse_thing)
            .transpose()?;

        let log = self
            .client
            .create("request_log")
            .content(RequestLogRecord {
                request_id: input.request_id,
                user,
                virtual_api_key,
                model_alias: input.model_alias,
                provider: input.provider,
                upstream_model: input.upstream_model,
                status_code: input.status_code,
                latency_ms: input.latency_ms,
                stream: input.stream,
                request_url: input.request_url,
                error_message: input.error_message,
                usage_input_tokens: input.usage_input_tokens,
                usage_output_tokens: input.usage_output_tokens,
            })
            .await?;

        Ok(log)
    }

    pub fn decrypt_provider_api_key(
        &self,
        encrypted_api_key: &str,
    ) -> Result<String, DatabaseError> {
        decrypt_secret(&self.encryption_key, encrypted_api_key)
    }

    async fn ensure_model_aliases_exist(&self, aliases: &[String]) -> Result<(), DatabaseError> {
        if aliases.is_empty() {
            return Ok(());
        }

        let existing_aliases = self
            .client
            .query(
                "SELECT VALUE alias FROM model_catalog
                 WHERE alias INSIDE $aliases
                   AND enabled = true;",
            )
            .bind(("aliases", aliases.to_vec()))
            .await?
            .take::<Vec<String>>(0)?;

        let existing_aliases: std::collections::HashSet<_> = existing_aliases.into_iter().collect();
        let missing_aliases: Vec<_> = aliases
            .iter()
            .filter(|alias| !existing_aliases.contains(alias.as_str()))
            .cloned()
            .collect();

        if !missing_aliases.is_empty() {
            return Err(DatabaseError::InvalidConfig(format!(
                "model alias(es) do not exist or are disabled: {}",
                missing_aliases.join(", ")
            )));
        }

        Ok(())
    }

    async fn seed_model_catalog(&self) -> Result<(), DatabaseError> {
        for (record_id, record) in model_catalog_seeds() {
            self.client
                .query("UPSERT type::record('model_catalog', $id) CONTENT $record;")
                .bind((
                    "id",
                    record_id
                        .strip_prefix("model_catalog:")
                        .unwrap_or(record_id),
                ))
                .bind(("record", record))
                .await?;
        }

        Ok(())
    }

    async fn fresh_client(&self) -> Result<Surreal<any::Any>, DatabaseError> {
        let client = any::connect(&self.config.surreal_url).await?;
        client
            .use_ns(&self.config.surreal_namespace)
            .use_db(&self.config.surreal_database)
            .await?;

        Ok(client)
    }

    async fn fresh_client_with_token(
        &self,
        token: &str,
    ) -> Result<Surreal<any::Any>, DatabaseError> {
        let client = self.fresh_client().await?;
        client.authenticate(token).await?;
        Ok(client)
    }
}

fn parse_thing(value: &str) -> Result<RecordId, DatabaseError> {
    RecordId::parse_simple(value)
        .map_err(|_| DatabaseError::InvalidConfig(format!("invalid record id: {value}")))
}
