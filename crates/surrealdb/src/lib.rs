//! `SurrealDB` facade for `ValyMux`.
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
    ModelCatalogEntry, ModelSyncInput, ProviderCredential, ProviderKind, RequestLog,
    RequestLogInput, ResolvedProxyRoute, SigninInput, SignupInput, UpdateProfileInput,
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

        Ok(())
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
            .await
            .map_err(|e| {
                let msg = e.to_string().to_lowercase();
                if msg.contains("signup") && msg.contains("query failed") {
                    DatabaseError::InvalidConfig("an account with that email already exists".into())
                } else {
                    DatabaseError::from(e)
                }
            })?;

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
            .await
            .map_err(|e| {
                let msg = e.to_string().to_lowercase();
                if msg.contains("no record was returned") {
                    DatabaseError::InvalidConfig("invalid email or password".into())
                } else {
                    DatabaseError::from(e)
                }
            })?;

        let user = user_client
            .query("SELECT * FROM $auth.id;")
            .await?
            .take::<Option<User>>(0)?
            .ok_or_else(|| DatabaseError::SchemaBootstrap("signed in user was not found".into()))?;

        if !user.enabled {
            return Err(DatabaseError::InvalidConfig("account is disabled".into()));
        }

        Ok(AuthSession {
            user,
            token: token.access.into_insecure_token(),
        })
    }

    pub async fn authenticate_user(&self, token: &str) -> Result<User, DatabaseError> {
        let client = self.fresh_client_with_token(token).await?;
        let user = client
            .query("SELECT * FROM $auth.id;")
            .await?
            .take::<Option<User>>(0)?
            .ok_or_else(|| DatabaseError::InvalidConfig("authenticated user not found".into()))?;

        if !user.enabled {
            return Err(DatabaseError::InvalidConfig("account is disabled".into()));
        }

        Ok(user)
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
                    enabled: true,
                    sync_status: 'pending',
                    model_count: 0
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
                "UPDATE $provider_id MERGE {
                    label: $label,
                    tags: $tags,
                    enabled: $enabled,
                    updated_at: time::now()
                };",
            )
            .bind(("provider_id", provider_id.clone()))
            .bind(("label", input.label))
            .bind(("tags", input.tags))
            .bind(("enabled", input.enabled))
            .await?;

        // Update encrypted_api_key separately only when a new key is provided
        if let Some(key) = encrypted_api_key {
            client
                .query("UPDATE $provider_id MERGE { encrypted_api_key: $encrypted_api_key };")
                .bind(("provider_id", provider_id.clone()))
                .bind(("encrypted_api_key", key))
                .await?;
        }

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

        // Delete all model_catalog entries for this credential (root client — bypasses permissions)
        self.client
            .query("DELETE model_catalog WHERE provider_credential = $cred_id;")
            .bind(("cred_id", provider_id.clone()))
            .await?;

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
        let user = self.authenticate_user(token).await?;
        self.ensure_model_aliases_exist_for_user(&user.id, &input.allowed_models)
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
        let user = self.authenticate_user(token).await?;
        self.ensure_model_aliases_exist_for_user(&user.id, &input.allowed_models)
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

    /// Lists models from the user's discovered catalog (across all their credentials).
    pub async fn list_usable_models(
        &self,
        token: &str,
    ) -> Result<Vec<ModelCatalogEntry>, DatabaseError> {
        let user = self.authenticate_user(token).await?;
        self.client
            .query(
                "SELECT * FROM model_catalog
                 WHERE user = $user_id
                   AND enabled = true
                   AND provider_credential.enabled = true
                 ORDER BY alias ASC;",
            )
            .bind(("user_id", user.id))
            .await?
            .take::<Vec<ModelCatalogEntry>>(0)
            .map_err(Into::into)
    }

    /// Gets a single model from the user's catalog by alias.
    pub async fn get_model_by_alias_for_user(
        &self,
        token: &str,
        alias: &str,
    ) -> Result<Option<ModelCatalogEntry>, DatabaseError> {
        let user = self.authenticate_user(token).await?;
        self.client
            .query(
                "SELECT * FROM model_catalog
                 WHERE user = $user_id
                   AND alias = $alias
                   AND enabled = true
                 LIMIT 1;",
            )
            .bind(("user_id", user.id))
            .bind(("alias", alias.to_string()))
            .await?
            .take::<Option<ModelCatalogEntry>>(0)
            .map_err(Into::into)
    }

    /// Lists models discovered for a specific credential. Uses root client — caller must verify
    /// ownership at the handler level.
    pub async fn list_models_for_credential(
        &self,
        credential_id: &str,
    ) -> Result<Vec<ModelCatalogEntry>, DatabaseError> {
        let cred_id = parse_thing(credential_id)?;
        self.client
            .query(
                "SELECT * FROM model_catalog
                 WHERE provider_credential = $cred_id
                   AND enabled = true
                 ORDER BY alias ASC;",
            )
            .bind(("cred_id", cred_id))
            .await?
            .take::<Vec<ModelCatalogEntry>>(0)
            .map_err(Into::into)
    }

    /// Fetches a provider credential using the root client. Used by background sync tasks where
    /// no user token is available.
    pub async fn get_credential_for_sync(
        &self,
        credential_id: &str,
    ) -> Result<Option<ProviderCredential>, DatabaseError> {
        let id = parse_thing(credential_id)?;
        self.client
            .query("SELECT * FROM $id;")
            .bind(("id", id))
            .await?
            .take::<Option<ProviderCredential>>(0)
            .map_err(Into::into)
    }

    /// Updates sync status on a provider credential. Uses root client (background task context).
    pub async fn set_credential_sync_status(
        &self,
        credential_id: &str,
        status: &str,
        error: Option<String>,
    ) -> Result<(), DatabaseError> {
        let id = parse_thing(credential_id)?;
        self.client
            .query(
                "UPDATE $id MERGE {
                    sync_status: $status,
                    sync_error: $error,
                    updated_at: time::now()
                };",
            )
            .bind(("id", id))
            .bind(("status", status.to_string()))
            .bind(("error", error))
            .await?;
        Ok(())
    }

    /// Replaces the model catalog for a credential: deletes all existing entries then batch-inserts
    /// the newly discovered models. Uses root client (background task context).
    pub async fn sync_models(
        &self,
        credential_id: &str,
        user_id: RecordId,
        models: Vec<ModelSyncInput>,
    ) -> Result<i64, DatabaseError> {
        let cred_id = parse_thing(credential_id)?;

        // Delete existing models for this credential
        self.client
            .query("DELETE model_catalog WHERE provider_credential = $cred_id;")
            .bind(("cred_id", cred_id.clone()))
            .await?;

        let count = i64::try_from(models.len()).unwrap_or(i64::MAX);

        // Insert each discovered model
        for model in models {
            self.client
                .query(
                    "CREATE model_catalog CONTENT {
                        user: $user_id,
                        provider_credential: $cred_id,
                        alias: $alias,
                        provider: $provider,
                        upstream_model: $upstream_model,
                        display_name: $display_name,
                        description: $description,
                        context_window_tokens: $context_window_tokens,
                        max_output_tokens: $max_output_tokens,
                        supports_streaming: $supports_streaming,
                        supports_thinking: $supports_thinking,
                        thinking_required: $thinking_required,
                        supports_temperature: $supports_temperature,
                        temperature_fixed_to: $temperature_fixed_to,
                        temperature_min: $temperature_min,
                        temperature_max: $temperature_max,
                        supports_top_p: $supports_top_p,
                        supports_system_messages: $supports_system_messages,
                        supports_tools: $supports_tools,
                        supports_vision: $supports_vision,
                        supports_json_mode: $supports_json_mode,
                        supports_parallel_tool_calls: $supports_parallel_tool_calls,
                        enabled: true
                    };",
                )
                .bind(("user_id", user_id.clone()))
                .bind(("cred_id", cred_id.clone()))
                .bind(("alias", model.alias))
                .bind(("provider", model.provider))
                .bind(("upstream_model", model.upstream_model))
                .bind(("display_name", model.display_name))
                .bind(("description", model.description))
                .bind(("context_window_tokens", model.context_window_tokens))
                .bind(("max_output_tokens", model.max_output_tokens))
                .bind(("supports_streaming", model.supports_streaming))
                .bind(("supports_thinking", model.supports_thinking))
                .bind(("thinking_required", model.thinking_required))
                .bind(("supports_temperature", model.supports_temperature))
                .bind(("temperature_fixed_to", model.temperature_fixed_to))
                .bind(("temperature_min", model.temperature_min))
                .bind(("temperature_max", model.temperature_max))
                .bind(("supports_top_p", model.supports_top_p))
                .bind(("supports_system_messages", model.supports_system_messages))
                .bind(("supports_tools", model.supports_tools))
                .bind(("supports_vision", model.supports_vision))
                .bind(("supports_json_mode", model.supports_json_mode))
                .bind((
                    "supports_parallel_tool_calls",
                    model.supports_parallel_tool_calls,
                ))
                .await?;
        }

        // Update model count and last_synced_at on the credential
        self.client
            .query(
                "UPDATE $cred_id MERGE {
                    model_count: $count,
                    last_synced_at: time::now(),
                    sync_status: 'synced',
                    sync_error: NONE,
                    updated_at: time::now()
                };",
            )
            .bind(("cred_id", cred_id))
            .bind(("count", count))
            .await?;

        Ok(count)
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

        // Empty allowed_models = unrestricted (key can use any model the user has access to)
        if !verified.key.allowed_models.is_empty()
            && !verified
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

        // Find model in the user's personal catalog (not global)
        let model = self
            .client
            .query(
                "SELECT * FROM model_catalog
                 WHERE user = $user_id
                   AND alias = $alias
                   AND enabled = true
                 LIMIT 1;",
            )
            .bind(("user_id", user.id.clone()))
            .bind(("alias", requested_model))
            .await?
            .take::<Option<ModelCatalogEntry>>(0)?
            .ok_or_else(|| {
                DatabaseError::NotFound("requested model was not found in your catalog".into())
            })?;

        // Fetch the exact provider credential that discovered this model (direct FK)
        let provider_credential = self
            .client
            .query("SELECT * FROM $cred_id;")
            .bind(("cred_id", model.provider_credential.clone()))
            .await?
            .take::<Option<ProviderCredential>>(0)?
            .ok_or_else(|| {
                DatabaseError::NotFound("provider credential for this model was not found".into())
            })?;

        if !provider_credential.enabled {
            return Err(DatabaseError::NotFound(
                "provider credential for this model is disabled".into(),
            ));
        }

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

    /// Validates that all provided model aliases exist in the given user's discovered catalog.
    async fn ensure_model_aliases_exist_for_user(
        &self,
        user_id: &RecordId,
        aliases: &[String],
    ) -> Result<(), DatabaseError> {
        if aliases.is_empty() {
            return Ok(());
        }

        let existing_aliases = self
            .client
            .query(
                "SELECT VALUE alias FROM model_catalog
                 WHERE user = $user_id
                   AND alias INSIDE $aliases
                   AND enabled = true;",
            )
            .bind(("user_id", user_id.clone()))
            .bind(("aliases", aliases.to_vec()))
            .await?
            .take::<Vec<String>>(0)?;

        let existing_set: std::collections::HashSet<_> = existing_aliases.into_iter().collect();
        let missing: Vec<_> = aliases
            .iter()
            .filter(|alias| !existing_set.contains(alias.as_str()))
            .cloned()
            .collect();

        if !missing.is_empty() {
            return Err(DatabaseError::InvalidConfig(format!(
                "model alias(es) not found in your catalog: {}",
                missing.join(", ")
            )));
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
