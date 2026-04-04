//! `SurrealDB` facade for `ValyMux`.
//!
//! This crate provides a secure database layer with two authentication planes:
//!
//! **Control Plane (Dashboard):**
//! - User signs in → JWT token → `client_with_jwt(token)` → call user-scoped functions
//! - All CRUD operations use `$auth_id` for row-level security
//!
//! **Data Plane (LLM Proxy):**
//! - Virtual API key → `client_with_virtual_key(raw_key)` → proxy-scoped route + log functions
//! - Backend service grant → persistent authenticated client → encrypted secret retrieval
//! - No dashboard JWT required - authorization is done by authenticating as the virtual key
//!
//! **Security:**
//! - NO root client is stored in this struct
//! - User operations use JWT auth, proxy routing uses virtual-key auth, and provider secrets are
//!   fetched only through the backend service grant
//! - Schema is applied manually via Surrealist/CLI, never by the server

mod config;
mod crypto;
mod error;
mod models;

use std::sync::Arc;

use serde::Serialize;
use surrealdb::{
    Surreal,
    engine::any,
    opt::auth::{Record, Token},
    types::{RecordId, SurrealValue},
};
use tokio::sync::OnceCell;

pub use config::DatabaseConfig;
use crypto::{
    decrypt_secret, encrypt_secret, generate_virtual_api_key, hash_virtual_api_key, key_prefix,
};
pub use error::DatabaseError;
pub use models::{
    AuthSession, CreateProviderCredentialInput, CreateVirtualApiKeyInput, CreatedVirtualApiKey,
    ModelDefinition, ModelSyncInput, ProviderCredential, ProviderKind, RequestLog, RequestLogInput,
    ResolvedProxyRoute, RouteModelInfo, SigninInput, SignupInput, UpdateProfileInput,
    UpdateProviderCredentialInput, UpdateVirtualApiKeyInput, User, VirtualApiKey, VirtualKeyRoute,
    VirtualKeyRouteInput,
};

/// Database facade with NO root client.
/// All operations use either JWT-authenticated clients or virtual-key authenticated clients.
#[derive(Clone)]
pub struct Database {
    config: DatabaseConfig,
    encryption_key: [u8; 32],
    service_client: Arc<OnceCell<Surreal<any::Any>>>,
}

pub struct ProxySession {
    client: Surreal<any::Any>,
}

#[allow(clippy::missing_errors_doc)]
impl Database {
    /// Creates a new Database instance.
    /// Note: This does NOT connect to the database or run any queries.
    /// Schema must be applied manually via Surrealist/CLI before using this.
    pub fn new(config: DatabaseConfig) -> Result<Self, DatabaseError> {
        config.validate()?;
        let encryption_key = config.encryption_key_bytes()?;

        Ok(Self {
            config,
            encryption_key,
            service_client: Arc::new(OnceCell::new()),
        })
    }

    /// Connects and authenticates the backend-service client once at startup.
    pub async fn initialize_runtime(&self) -> Result<(), DatabaseError> {
        if !self.config.has_service_credentials() {
            return Err(DatabaseError::InvalidConfig(
                "VALYMUX_DB_SERVICE_KEY is required but not set. \
                 Run `./scripts/generate_backend_grant.sh` to create the backend service grant, \
                 then set the returned key as VALYMUX_DB_SERVICE_KEY in your .env file."
                    .into(),
            ));
        }
        let _ = self.service_client().await.map_err(|e| {
            DatabaseError::ServiceAuth(format!(
                "Backend service authentication failed: {e}. \
                 If the grant has expired (30-day TTL), run `./scripts/generate_backend_grant.sh` \
                 to rotate the key."
            ))
        })?;
        Ok(())
    }

    // ==================== Auth Operations ====================

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

        // After signup, the client is authenticated - extract user id from JWT and call fn::get_current_user
        let token_str = token.access.into_insecure_token();
        let user = user_client
            .query("RETURN fn::get_current_user();")
            .await?
            .take::<Option<User>>(0)?
            .ok_or_else(|| DatabaseError::NotFound("signed up user was not found".into()))?;

        Ok(AuthSession {
            user,
            token: token_str,
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

        // After signin, extract user id from JWT and call fn::get_current_user
        // This also validates that the user is enabled
        let token_str = token.access.into_insecure_token();
        let user = user_client
            .query("RETURN fn::get_current_user();")
            .await?
            .take::<Option<User>>(0)?
            .ok_or_else(|| DatabaseError::NotFound("signed in user was not found".into()))?;

        Ok(AuthSession {
            user,
            token: token_str,
        })
    }

    pub async fn authenticate_user(&self, token: &str) -> Result<User, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        client
            .query("RETURN fn::get_current_user();")
            .await?
            .take::<Option<User>>(0)?
            .ok_or_else(|| DatabaseError::NotFound("authenticated user not found".into()))
    }

    pub async fn update_profile(
        &self,
        token: &str,
        input: UpdateProfileInput,
    ) -> Result<User, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        client
            .query("RETURN fn::update_profile($changed_name); RETURN fn::get_current_user();")
            .bind(("changed_name", input.name))
            .await?
            // Index 1 = fn::get_current_user() (the freshly-updated user); index 0 = fn::update_profile return (ignored)
            .take::<Option<User>>(1)?
            .ok_or_else(|| DatabaseError::NotFound("user not found after profile update".into()))
    }

    // ==================== Provider Credential Operations ====================

    pub async fn list_provider_credentials(
        &self,
        token: &str,
    ) -> Result<Vec<ProviderCredential>, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        client
            .query("RETURN fn::list_provider_credentials();")
            .await?
            .take::<Vec<ProviderCredential>>(0)
            .map_err(Into::into)
    }

    pub async fn get_provider_credential(
        &self,
        token: &str,
        credential_id: &str,
    ) -> Result<Option<ProviderCredential>, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        let cred_id = parse_thing(credential_id)?;
        client
            .query("RETURN fn::get_provider_credential($id);")
            .bind(("id", cred_id))
            .await?
            .take::<Option<ProviderCredential>>(0)
            .map_err(Into::into)
    }

    pub async fn create_provider_credential(
        &self,
        token: &str,
        input: CreateProviderCredentialInput,
    ) -> Result<ProviderCredential, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        let encrypted_api_key = encrypt_secret(&self.encryption_key, &input.api_key)?;

        client
            .query("RETURN fn::create_provider_credential($provider, $label, $encrypted_api_key, $tags);")
            .bind(("provider", input.provider.as_str().to_string()))
            .bind(("label", input.label))
            .bind(("encrypted_api_key", encrypted_api_key))
            .bind(("tags", input.tags))
            .await?
            .take::<Option<ProviderCredential>>(0)?
            .ok_or_else(|| {
                DatabaseError::NotFound("provider credential creation returned no record".into())
            })
    }

    pub async fn update_provider_credential(
        &self,
        token: &str,
        credential_id: &str,
        input: UpdateProviderCredentialInput,
    ) -> Result<Option<ProviderCredential>, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        let cred_id = parse_thing(credential_id)?;
        let encrypted_api_key = match input.api_key {
            Some(api_key) => Some(encrypt_secret(&self.encryption_key, &api_key)?),
            None => None,
        };

        client
            .query("RETURN fn::update_provider_credential($id, $label, $tags, $enabled, $encrypted_api_key);")
            .bind(("id", cred_id))
            .bind(("label", input.label))
            .bind(("tags", input.tags))
            .bind(("enabled", input.enabled))
            .bind(("encrypted_api_key", encrypted_api_key))
            .await?
            .take::<Option<ProviderCredential>>(0)
            .map_err(Into::into)
    }

    pub async fn delete_provider_credential(
        &self,
        token: &str,
        credential_id: &str,
    ) -> Result<Option<ProviderCredential>, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        let cred_id = parse_thing(credential_id)?;

        // The function handles cascade delete of model_catalog entries
        client
            .query("RETURN fn::delete_provider_credential($id);")
            .bind(("id", cred_id))
            .await?
            .take::<Option<ProviderCredential>>(0)
            .map_err(Into::into)
    }

    // ==================== Virtual API Key Operations ====================

    pub async fn list_virtual_api_keys(
        &self,
        token: &str,
    ) -> Result<Vec<VirtualApiKey>, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        client
            .query("RETURN fn::list_virtual_api_keys();")
            .await?
            .take::<Vec<VirtualApiKey>>(0)
            .map_err(Into::into)
    }

    pub async fn get_virtual_api_key(
        &self,
        token: &str,
        key_id: &str,
    ) -> Result<Option<VirtualApiKey>, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        let key_id = parse_thing(key_id)?;
        client
            .query("RETURN fn::get_virtual_api_key($id);")
            .bind(("id", key_id))
            .await?
            .take::<Option<VirtualApiKey>>(0)
            .map_err(Into::into)
    }

    pub async fn create_virtual_api_key(
        &self,
        token: &str,
        input: CreateVirtualApiKeyInput,
    ) -> Result<CreatedVirtualApiKey, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        let raw_key = generate_virtual_api_key()?;

        let created = client
            .query("RETURN fn::create_virtual_api_key($name, $key_prefix, $key_hash, $allowed_models, $tags, $expires_at, $routes);")
            .bind(("name", input.name))
            .bind(("key_prefix", key_prefix(&raw_key)))
            .bind(("key_hash", hash_virtual_api_key(&raw_key)))
            .bind(("allowed_models", input.allowed_models))
            .bind(("tags", input.tags))
            .bind(("expires_at", input.expires_at))
            .bind(("routes", input.routes))
            .await?
            .take::<Option<VirtualApiKey>>(0)?
            .ok_or_else(|| {
                DatabaseError::NotFound("virtual API key creation returned no record".into())
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
        let client = self.client_with_jwt(token).await?;
        let key_id = parse_thing(key_id)?;

        client
            .query("RETURN fn::update_virtual_api_key($id, $name, $allowed_models, $tags, $enabled, $expires_at, $routes);")
            .bind(("id", key_id))
            .bind(("name", input.name))
            .bind(("allowed_models", input.allowed_models))
            .bind(("tags", input.tags))
            .bind(("enabled", input.enabled))
            .bind(("expires_at", input.expires_at))
            .bind(("routes", input.routes))
            .await?
            .take::<Option<VirtualApiKey>>(0)
            .map_err(Into::into)
    }

    pub async fn delete_virtual_api_key(
        &self,
        token: &str,
        key_id: &str,
    ) -> Result<Option<VirtualApiKey>, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        let key_id = parse_thing(key_id)?;
        client
            .query("RETURN fn::delete_virtual_api_key($id);")
            .bind(("id", key_id))
            .await?
            .take::<Option<VirtualApiKey>>(0)
            .map_err(Into::into)
    }

    // ==================== Model Catalog Operations ====================

    /// Lists models from the user's discovered catalog (across all their credentials).
    pub async fn list_usable_models(
        &self,
        token: &str,
    ) -> Result<Vec<ModelDefinition>, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        client
            .query("RETURN fn::list_usable_models();")
            .await?
            .take::<Vec<ModelDefinition>>(0)
            .map_err(Into::into)
    }

    /// Gets a single model from the user's catalog by alias.
    pub async fn get_model_by_alias(
        &self,
        token: &str,
        alias: &str,
    ) -> Result<Option<ModelDefinition>, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        client
            .query("RETURN fn::get_model_by_alias($alias);")
            .bind(("alias", alias.to_string()))
            .await?
            .take::<Option<ModelDefinition>>(0)
            .map_err(Into::into)
    }

    /// Lists models discovered for a specific credential.
    pub async fn list_models_for_credential(
        &self,
        token: &str,
        credential_id: &str,
    ) -> Result<Vec<ModelDefinition>, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        let cred_id = parse_thing(credential_id)?;
        client
            .query("RETURN fn::list_models_for_credential($cred_id);")
            .bind(("cred_id", cred_id))
            .await?
            .take::<Vec<ModelDefinition>>(0)
            .map_err(Into::into)
    }

    /// Syncs models for a credential: deletes existing, inserts new.
    /// Called immediately after credential creation while JWT is available.
    pub async fn sync_models(
        &self,
        token: &str,
        credential_id: &str,
        models: Vec<ModelSyncInput>,
    ) -> Result<i64, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        let cred_id = parse_thing(credential_id)?;

        // Retry on transaction conflicts (can occur when two syncs race on the same credential).
        // SurrealDB itself signals these as retryable: "Transaction conflict: Resource busy".
        for attempt in 0u32..3 {
            let result = client
                .query("RETURN fn::sync_models($cred_id, $models);")
                .bind(("cred_id", cred_id.clone()))
                .bind(("models", models.clone()))
                .await
                .and_then(|mut r| r.take::<Option<i64>>(0));

            match result {
                Ok(Some(count)) => return Ok(count),
                Ok(None) => {
                    return Err(DatabaseError::NotFound(
                        "sync_models returned no count".into(),
                    ));
                }
                Err(e) if attempt < 2 && e.to_string().contains("Transaction conflict") => {
                    let delay = std::time::Duration::from_millis(200 * 2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                }
                Err(e) => return Err(e.into()),
            }
        }
        // Safety: the final attempt (attempt == 2) always returns via `Err(e) => return Err(...)`.
        // All three attempts either return immediately (Ok variants) or exhaust retries and return.
        unreachable!()
    }

    /// Updates sync status on a credential (for error reporting).
    pub async fn set_credential_sync_status(
        &self,
        token: &str,
        credential_id: &str,
        status: &str,
        error: Option<String>,
    ) -> Result<(), DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        let cred_id = parse_thing(credential_id)?;

        client
            .query("RETURN fn::set_credential_sync_status($cred_id, $status, $error);")
            .bind(("cred_id", cred_id))
            .bind(("status", status.to_string()))
            .bind(("error", error))
            .await?;

        Ok(())
    }

    /// Atomically transitions a credential to "syncing" only if it is not already syncing.
    /// Returns `true` if the caller acquired the sync lock, `false` if already syncing.
    pub async fn start_credential_sync(
        &self,
        token: &str,
        credential_id: &str,
    ) -> Result<bool, DatabaseError> {
        let client = self.client_with_jwt(token).await?;
        let cred_id = parse_thing(credential_id)?;
        client
            .query("RETURN fn::start_provider_sync($id);")
            .bind(("id", cred_id))
            .await?
            .take::<Option<bool>>(0)
            .map_err(Into::into)
            .map(|v| v.unwrap_or(false))
    }

    // ==================== Proxy Operations (Data Plane) ====================

    /// Authenticates the database client as a virtual API key for one proxy request.
    pub async fn begin_proxy_session(&self, raw_key: &str) -> Result<ProxySession, DatabaseError> {
        let client = self.client_with_virtual_key(raw_key).await?;
        Ok(ProxySession { client })
    }

    pub async fn fetch_proxy_provider_api_key(
        &self,
        credential_id: &RecordId,
    ) -> Result<String, DatabaseError> {
        if !self.config.has_service_credentials() {
            return Err(DatabaseError::InvalidConfig(
                "proxy secret fetching requires VALYMUX_DB_SERVICE_KEY".into(),
            ));
        }

        let encrypted_api_key = self
            .service_client()
            .await?
            .query("RETURN fn::backend_fetch_secret($cred_id);")
            .bind(("cred_id", credential_id.clone()))
            .await
            .map_err(|error| DatabaseError::SecretFetch(error.to_string()))?
            .take::<Option<String>>(0)
            .map_err(|error| DatabaseError::SecretFetch(error.to_string()))?
            .ok_or_else(|| {
                DatabaseError::SecretFetch(
                    "backend_fetch_secret returned no encrypted API key".into(),
                )
            })?;

        decrypt_secret(&self.encryption_key, &encrypted_api_key)
    }

    // ==================== Utility ====================

    pub fn decrypt_provider_api_key(
        &self,
        encrypted_api_key: &str,
    ) -> Result<String, DatabaseError> {
        decrypt_secret(&self.encryption_key, encrypted_api_key)
    }

    // ==================== Internal Helpers ====================

    /// Creates an unauthenticated client connection.
    /// Used for signup/signin flows before an authenticated token exists.
    async fn fresh_client(&self) -> Result<Surreal<any::Any>, DatabaseError> {
        Self::fresh_client_from_config(&self.config).await
    }

    async fn fresh_client_from_config(
        config: &DatabaseConfig,
    ) -> Result<Surreal<any::Any>, DatabaseError> {
        let client = any::connect(&config.surreal_url).await?;
        client
            .use_ns(&config.surreal_namespace)
            .use_db(&config.surreal_database)
            .await?;

        Ok(client)
    }

    async fn service_client(&self) -> Result<&Surreal<any::Any>, DatabaseError> {
        self.service_client
            .get_or_try_init(|| async {
                #[derive(Serialize, SurrealValue)]
                #[surreal(crate = "surrealdb::types")]
                struct Params {
                    key: String,
                }

                let client = Self::fresh_client_from_config(&self.config).await?;
                let _: Token = client
                    .signin(Record {
                        access: "backend_service".to_string(),
                        namespace: self.config.surreal_namespace.clone(),
                        database: self.config.surreal_database.clone(),
                        params: Params {
                            key: self.config.surreal_service_key.clone(),
                        },
                    })
                    .await
                    .map_err(|error| DatabaseError::ServiceAuth(error.to_string()))?;
                Ok(client)
            })
            .await
    }

    /// Creates a JWT-authenticated client connection and extracts the user RecordId.
    /// Used for all control plane operations (dashboard CRUD).
    async fn client_with_jwt(&self, token: &str) -> Result<Surreal<any::Any>, DatabaseError> {
        let client = self.fresh_client().await?;
        client.authenticate(token).await?;
        Ok(client)
    }

    /// Creates a virtual-key authenticated client for the proxy data plane.
    async fn client_with_virtual_key(
        &self,
        raw_key: &str,
    ) -> Result<Surreal<any::Any>, DatabaseError> {
        #[derive(Serialize, SurrealValue)]
        #[surreal(crate = "surrealdb::types")]
        struct Params {
            virtual_key: String,
        }

        let client = self.fresh_client().await?;
        let _: Token = client
            .signin(Record {
                access: "virtual_key_access".to_string(),
                namespace: self.config.surreal_namespace.clone(),
                database: self.config.surreal_database.clone(),
                params: Params {
                    virtual_key: raw_key.to_string(),
                },
            })
            .await?;
        Ok(client)
    }
}

#[allow(clippy::missing_errors_doc)]
impl ProxySession {
    pub async fn resolve_route(
        &self,
        requested_model: &str,
    ) -> Result<ResolvedProxyRoute, DatabaseError> {
        self.client
            .query("RETURN fn::proxy_resolve_route($model_alias);")
            .bind(("model_alias", requested_model.to_string()))
            .await?
            .take::<Option<ResolvedProxyRoute>>(0)?
            .ok_or_else(|| DatabaseError::NotFound("proxy route resolution failed".into()))
    }

    pub async fn log_request(
        &self,
        input: RequestLogInput,
    ) -> Result<Option<RequestLog>, DatabaseError> {
        self.client
            .query(
                "RETURN fn::proxy_log_request($request_id, $model_alias, $provider, \
                 $upstream_model, $status_code, $latency_ms, $stream, $request_url, \
                 $error_message, $usage_input_tokens, $usage_output_tokens);",
            )
            .bind(("request_id", input.request_id))
            .bind(("model_alias", input.model_alias))
            .bind(("provider", input.provider))
            .bind(("upstream_model", input.upstream_model))
            .bind(("status_code", input.status_code))
            .bind(("latency_ms", input.latency_ms))
            .bind(("stream", input.stream))
            .bind(("request_url", input.request_url))
            .bind(("error_message", input.error_message))
            .bind(("usage_input_tokens", input.usage_input_tokens))
            .bind(("usage_output_tokens", input.usage_output_tokens))
            .await?
            .take::<Option<RequestLog>>(0)
            .map_err(Into::into)
    }
}

fn parse_thing(value: &str) -> Result<RecordId, DatabaseError> {
    RecordId::parse_simple(value)
        .map_err(|_| DatabaseError::InvalidConfig(format!("invalid record id: {value}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

    fn user_id_from_token(token: &str) -> Result<RecordId, DatabaseError> {
        let parts: Vec<&str> = token.splitn(3, '.').collect();
        if parts.len() != 3 {
            return Err(DatabaseError::InvalidConfig("invalid JWT format".into()));
        }
        let payload = URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|_| DatabaseError::InvalidConfig("invalid JWT payload encoding".into()))?;
        let json: serde_json::Value = serde_json::from_slice(&payload)
            .map_err(|_| DatabaseError::InvalidConfig("invalid JWT payload JSON".into()))?;
        let id_str = json
            .get("ID")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DatabaseError::InvalidConfig("JWT missing ID claim".into()))?;
        parse_thing(id_str)
    }

    fn test_config() -> DatabaseConfig {
        DatabaseConfig {
            surreal_url: "mem://".into(),
            surreal_namespace: "test".into(),
            surreal_database: "test".into(),
            surreal_username: None,
            surreal_password: None,
            surreal_service_key: "service-key".into(),
            surreal_encryption_key:
                "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".into(),
        }
    }

    #[test]
    fn test_database_new_succeeds_with_valid_config() {
        let db = Database::new(test_config());
        assert!(db.is_ok(), "Database::new should succeed with valid config");
    }

    #[test]
    fn test_database_new_fails_with_empty_url() {
        let mut config = test_config();
        config.surreal_url = "".into();
        let db = Database::new(config);
        assert!(db.is_err(), "Database::new should fail with empty URL");
    }

    #[test]
    fn test_database_new_fails_with_empty_namespace() {
        let mut config = test_config();
        config.surreal_namespace = "".into();
        let db = Database::new(config);
        assert!(
            db.is_err(),
            "Database::new should fail with empty namespace"
        );
    }

    #[test]
    fn test_database_new_fails_with_empty_database() {
        let mut config = test_config();
        config.surreal_database = "".into();
        let db = Database::new(config);
        assert!(db.is_err(), "Database::new should fail with empty database");
    }

    #[test]
    fn test_database_new_fails_with_empty_encryption_key() {
        let mut config = test_config();
        config.surreal_encryption_key = "".into();
        let db = Database::new(config);
        assert!(
            db.is_err(),
            "Database::new should fail with empty encryption key"
        );
    }

    #[test]
    fn test_parse_thing_valid() {
        let result = parse_thing("user:abc123");
        assert!(
            result.is_ok(),
            "parse_thing should succeed with valid record id"
        );
    }

    #[test]
    fn test_parse_thing_invalid() {
        let result = parse_thing("invalid");
        assert!(
            result.is_err(),
            "parse_thing should fail with invalid record id"
        );
    }

    #[test]
    fn test_decrypt_provider_api_key_roundtrip() {
        let db = Database::new(test_config()).unwrap();
        let original = "sk-test-api-key-12345";
        let encrypted = encrypt_secret(&db.encryption_key, original).unwrap();
        let decrypted = db.decrypt_provider_api_key(&encrypted).unwrap();
        assert_eq!(original, decrypted, "decrypted key should match original");
    }

    #[test]
    fn test_user_id_from_token_valid() {
        let payload =
            URL_SAFE_NO_PAD.encode(r#"{"iat":1,"nbf":1,"exp":9999999999,"ID":"user:abc123"}"#);
        let token = format!("eyJhbGciOiJIUzUxMiJ9.{payload}.fakesig");
        let result = user_id_from_token(&token);
        assert!(
            result.is_ok(),
            "should parse valid JWT with ID claim: {result:?}"
        );
    }

    #[test]
    fn test_user_id_from_token_missing_id() {
        let payload = URL_SAFE_NO_PAD.encode(r#"{"iat":1,"nbf":1,"exp":9999999999}"#);
        let token = format!("eyJhbGciOiJIUzUxMiJ9.{payload}.fakesig");
        let result = user_id_from_token(&token);
        assert!(result.is_err(), "should fail when ID claim is missing");
    }

    #[test]
    #[ignore = "manual diagnostic helper for decrypting a live ciphertext from env"]
    fn debug_decrypt_ciphertext_from_env() {
        let ciphertext = std::env::var("VALYMUX_DEBUG_CIPHERTEXT")
            .expect("VALYMUX_DEBUG_CIPHERTEXT must be set");
        let config = DatabaseConfig {
            surreal_url: std::env::var("SURREAL_URL").expect("SURREAL_URL must be set"),
            surreal_namespace: std::env::var("SURREAL_NAMESPACE")
                .expect("SURREAL_NAMESPACE must be set"),
            surreal_database: std::env::var("SURREAL_DATABASE")
                .expect("SURREAL_DATABASE must be set"),
            surreal_username: std::env::var("SURREAL_USERNAME").ok(),
            surreal_password: std::env::var("SURREAL_PASSWORD").ok(),
            surreal_service_key: std::env::var("VALYMUX_DB_SERVICE_KEY").unwrap_or_default(),
            surreal_encryption_key: std::env::var("SURREAL_ENCRYPTION_KEY")
                .expect("SURREAL_ENCRYPTION_KEY must be set"),
        };

        let db = Database::new(config).expect("database config should be valid");
        let plaintext = db
            .decrypt_provider_api_key(&ciphertext)
            .expect("ciphertext should decrypt");

        assert!(
            !plaintext.is_empty(),
            "decrypted provider secret should not be empty"
        );
    }

    #[tokio::test]
    #[ignore = "manual diagnostic helper for testing live backend secret fetch from env"]
    async fn debug_fetch_proxy_provider_api_key_from_env() {
        let provider_id = std::env::var("VALYMUX_DEBUG_PROVIDER_ID")
            .expect("VALYMUX_DEBUG_PROVIDER_ID must be set");
        let config = DatabaseConfig {
            surreal_url: std::env::var("SURREAL_URL").expect("SURREAL_URL must be set"),
            surreal_namespace: std::env::var("SURREAL_NAMESPACE")
                .expect("SURREAL_NAMESPACE must be set"),
            surreal_database: std::env::var("SURREAL_DATABASE")
                .expect("SURREAL_DATABASE must be set"),
            surreal_username: std::env::var("SURREAL_USERNAME").ok(),
            surreal_password: std::env::var("SURREAL_PASSWORD").ok(),
            surreal_service_key: std::env::var("VALYMUX_DB_SERVICE_KEY")
                .expect("VALYMUX_DB_SERVICE_KEY must be set"),
            surreal_encryption_key: std::env::var("SURREAL_ENCRYPTION_KEY")
                .expect("SURREAL_ENCRYPTION_KEY must be set"),
        };

        let db = Database::new(config).expect("database config should be valid");
        db.initialize_runtime()
            .await
            .expect("backend service client should initialize");

        let provider_id =
            parse_thing(&provider_id).expect("VALYMUX_DEBUG_PROVIDER_ID must be a record id");
        let plaintext = db
            .fetch_proxy_provider_api_key(&provider_id)
            .await
            .expect("provider secret fetch should succeed");

        assert!(
            !plaintext.is_empty(),
            "fetched provider secret should not be empty"
        );
    }
}
