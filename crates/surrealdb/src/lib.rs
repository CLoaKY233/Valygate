//! SurrealDB facade for ValyGate.
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
    opt::auth::{Jwt, Record, Root},
    sql::Thing,
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
                username: &config.surreal_username,
                password: &config.surreal_password,
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
        #[derive(Serialize)]
        struct Params<'a> {
            name: &'a str,
            email: &'a str,
            password: &'a str,
        }

        let user_client = self.fresh_client().await?;
        let token: Jwt = user_client
            .signup(Record {
                access: "account",
                namespace: &self.config.surreal_namespace,
                database: &self.config.surreal_database,
                params: Params {
                    name: &input.name,
                    email: &input.email,
                    password: &input.password,
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
            token: token.into_insecure_token(),
        })
    }

    pub async fn signin_user(&self, input: SigninInput) -> Result<AuthSession, DatabaseError> {
        #[derive(Serialize)]
        struct Params<'a> {
            email: &'a str,
            password: &'a str,
        }

        let user_client = self.fresh_client().await?;
        let token: Jwt = user_client
            .signin(Record {
                access: "account",
                namespace: &self.config.surreal_namespace,
                database: &self.config.surreal_database,
                params: Params {
                    email: &input.email,
                    password: &input.password,
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
            token: token.into_insecure_token(),
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
                UPDATE $provider_id MERGE
                    IF $encrypted_api_key = NONE {
                        $update
                    } ELSE {
                        $update.merge({ encrypted_api_key: $encrypted_api_key })
                    };",
            )
            .bind(("provider_id", provider_id.clone()))
            .bind(("label", input.label))
            .bind(("tags", input.tags))
            .bind(("enabled", input.enabled))
            .bind(("encrypted_api_key", encrypted_api_key))
            .await?;

        self.get_provider_credential(token, &provider_id.to_string())
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
        let raw_key = generate_virtual_api_key();

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

        self.get_virtual_api_key(token, &key_id.to_string()).await
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
        self.list_usable_models_for_user(&user.id.to_string()).await
    }

    pub async fn get_model_by_alias_for_user(
        &self,
        token: &str,
        alias: &str,
    ) -> Result<Option<ModelCatalogEntry>, DatabaseError> {
        let user = self.authenticate_user(token).await?;
        let models = self
            .list_usable_models_for_user(&user.id.to_string())
            .await?;
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
        #[derive(Serialize)]
        struct RequestLogRecord {
            request_id: String,
            user: Thing,
            virtual_api_key: Option<Thing>,
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
        for alias in aliases {
            let exists = self
                .client
                .query(
                    "SELECT VALUE count() > 0 FROM model_catalog
                     WHERE alias = $alias AND enabled = true
                     GROUP ALL;",
                )
                .bind(("alias", alias.clone()))
                .await?
                .take::<Option<bool>>(0)?
                .unwrap_or(false);

            if !exists {
                return Err(DatabaseError::InvalidConfig(format!(
                    "model alias `{alias}` does not exist or is disabled"
                )));
            }
        }

        Ok(())
    }

    async fn seed_model_catalog(&self) -> Result<(), DatabaseError> {
        #[derive(Serialize)]
        struct SeedModel<'a> {
            alias: &'a str,
            display_name: &'a str,
            provider: &'a str,
            upstream_model: &'a str,
            description: &'a str,
            tags: Vec<&'a str>,
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

        let seeds = [
            (
                "model_catalog:openai_gpt_4o_mini",
                SeedModel {
                    alias: "gpt-4o-mini",
                    display_name: "GPT-4o Mini",
                    provider: "openai",
                    upstream_model: "gpt-4o-mini",
                    description: "Fast low-cost OpenAI chat model.",
                    tags: vec!["chat", "fast"],
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
            ),
            (
                "model_catalog:openai_gpt_4o",
                SeedModel {
                    alias: "gpt-4o",
                    display_name: "GPT-4o",
                    provider: "openai",
                    upstream_model: "gpt-4o",
                    description: "General-purpose OpenAI flagship model.",
                    tags: vec!["chat", "flagship"],
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
            ),
            (
                "model_catalog:anthropic_claude_3_7_sonnet",
                SeedModel {
                    alias: "claude-3-7-sonnet",
                    display_name: "Claude 3.7 Sonnet",
                    provider: "anthropic",
                    upstream_model: "claude-3-7-sonnet-latest",
                    description: "Anthropic reasoning-capable chat model.",
                    tags: vec!["chat", "thinking"],
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
            ),
        ];

        for (record_id, record) in seeds {
            self.client
                .query("UPSERT type::thing('model_catalog', $id) CONTENT $record;")
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

fn parse_thing(value: &str) -> Result<Thing, DatabaseError> {
    surrealdb::sql::thing(value)
        .map_err(|_| DatabaseError::InvalidConfig(format!("invalid record id: {value}")))
}
