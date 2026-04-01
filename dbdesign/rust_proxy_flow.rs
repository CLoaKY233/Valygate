// ============================================================
// VALYMUX — RUST PROXY FLOW (pseudocode)
// ============================================================
//
// Two SurrealDB connections:
//   1. Per-request: authenticated via virtual_key_access (short-lived)
//   2. Persistent: authenticated via backend_service BEARER (long-lived)
//
// Connection 1 handles auth + routing (scoped, safe)
// Connection 2 handles secret retrieval (privileged, internal)
// ============================================================

// -- ONE-TIME SETUP (on deploy) ----------------------------------
//
// From a database owner session, generate the BEARER grant:
//
//   ACCESS backend_service ON DATABASE GRANT FOR USER;
//
// This returns a { key, secret } pair. Store them as:
//   VALYMUX_DB_SERVICE_KEY=...
//   VALYMUX_DB_SERVICE_SECRET=...
//
// The Rust backend authenticates with these at startup.
// This is NOT root — it's a database-scoped service credential.
// ----------------------------------------------------------------

use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Ws;

struct ProxyState {
    /// Per-request connections use virtual_key_access auth
    db_pool: SurrealPool, // your connection pool

    /// Persistent privileged connection for secret fetching
    service_db: Surreal<Ws>,
}

impl ProxyState {
    async fn init() -> Self {
        // 1. Normal pool for virtual_key_access connections
        let db_pool = SurrealPool::new("ws://surreal:8000").await;

        // 2. Privileged connection (backend_service BEARER)
        let service_db = Surreal::new::<Ws>("ws://surreal:8000").await.unwrap();
        service_db.use_ns("valymux").use_db("valymux").await.unwrap();
        service_db.signin(Bearer {
            key: std::env::var("VALYMUX_DB_SERVICE_KEY").unwrap(),
            secret: std::env::var("VALYMUX_DB_SERVICE_SECRET").unwrap(),
        }).await.unwrap();

        Self { db_pool, service_db }
    }
}

// -- PROXY REQUEST HANDLER ---------------------------------------

async fn handle_proxy_request(
    state: &ProxyState,
    virtual_key_raw: &str,  // from Authorization header
    model_alias: &str,      // from request path/body
) -> Result<UpstreamResponse, ProxyError> {

    // ── STEP 1: Authenticate virtual key ───────────────────────
    // Get a connection from pool, authenticate with virtual_key_access
    let db = state.db_pool.get().await?;
    db.signin(VirtualKeyAccess {
        virtual_key: virtual_key_raw.to_string(),
    }).await.map_err(|_| ProxyError::InvalidKey)?;

    // ── STEP 2: Resolve route (scoped, no secrets) ─────────────
    // This runs in virtual_key_access context.
    // Table permissions now allow reading model_catalog & provider_credential.
    // Field permission blocks encrypted_api_key — it comes back as NONE.
    let route: RouteInfo = db
        .run_fn("proxy_resolve_route", (model_alias,))
        .await
        .map_err(|e| ProxyError::RouteResolution(e.to_string()))?;

    // route contains:
    //   - virtual_key_id
    //   - user_id
    //   - model { alias, provider, upstream_model, capabilities... }
    //   - provider_credential_id  (record ID only, NO secret)
    //   - provider

    // ── STEP 3: Fetch secret (privileged, via service credential) ─
    // Uses the persistent backend_service connection.
    // System users bypass field permissions → gets encrypted_api_key.
    // fn::backend_fetch_secret has PERMISSIONS NONE, but system users
    // bypass function permissions too.
    let encrypted_key: String = state.service_db
        .run_fn("backend_fetch_secret", (route.provider_credential_id,))
        .await
        .map_err(|_| ProxyError::SecretFetch)?;

    // ── STEP 4: Decrypt provider key (in-memory only) ──────────
    let provider_api_key = aes_256_gcm_decrypt(
        &encrypted_key,
        &std::env::var("VALYMUX_ENCRYPTION_KEY").unwrap(),
    )?;

    // ── STEP 5: Forward to upstream provider ───────────────────
    let response = forward_to_provider(
        &route.provider,
        &route.model.upstream_model,
        &provider_api_key,
        request_body,
    ).await?;

    // ── STEP 6: Log the request (via virtual_key_access) ───────
    // No provider_credential in the log — just provider name.
    db.run_fn("proxy_log_request", (
        generate_request_id(),
        model_alias,
        &route.provider,
        &route.model.upstream_model,
        response.status_code,
        response.latency_ms,
        request_is_streaming,
        &request_url,
        response.error_message.as_deref(),
        response.usage_input_tokens,
        response.usage_output_tokens,
    )).await.ok(); // best-effort logging

    // provider_api_key is dropped here — never stored, never logged

    Ok(response)
}

// ── DATA STRUCTURES ────────────────────────────────────────────

#[derive(Deserialize)]
struct RouteInfo {
    virtual_key_id: Thing,
    user_id: Thing,
    model: ModelInfo,
    provider_credential_id: Thing,
    provider: String,
}

#[derive(Deserialize)]
struct ModelInfo {
    alias: String,
    provider: String,
    upstream_model: String,
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

// ── SECURITY BOUNDARIES ────────────────────────────────────────
//
//  WHAT EACH CONNECTION CAN DO:
//
//  ┌─────────────────────────┬─────────────────────────────────┐
//  │  virtual_key_access     │  backend_service (BEARER)       │
//  ├─────────────────────────┼─────────────────────────────────┤
//  │  ✓ Read own key record  │  ✓ Read encrypted_api_key       │
//  │  ✓ Read model_catalog   │  ✓ Any query (system user)      │
//  │    (owner's entries)    │                                 │
//  │  ✓ Read provider_cred   │  Used ONLY for:                │
//  │    (metadata only!)     │    fn::backend_fetch_secret()   │
//  │  ✗ encrypted_api_key    │                                 │
//  │  ✓ Create request_log   │  NEVER exposed to end users     │
//  │  ✗ Write anything else  │  NEVER used for user operations │
//  └─────────────────────────┴─────────────────────────────────┘
//
//  IF SOMEONE STEALS A VIRTUAL KEY AND CONNECTS DIRECTLY TO SURREALDB:
//    - They can read model catalog entries (aliases, capabilities) — this is NOT sensitive
//    - They can read provider credential metadata (label, provider name) — NOT sensitive
//    - They CANNOT read encrypted_api_key (field permission blocks it)
//    - They CANNOT call fn::backend_fetch_secret (PERMISSIONS NONE for record users)
//    - They CANNOT access other users' data ($auth.user scoping)
//    - The actual secret remains protected even in a direct-access scenario
//
