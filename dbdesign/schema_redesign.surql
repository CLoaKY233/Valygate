-- ============================================================
-- VALYMUX — FULL DATABASE SCHEMA REDESIGN
-- ============================================================
--
-- WHAT CHANGED & WHY:
--
-- PROBLEM: `PERMISSIONS FULL` on a function controls who can CALL it,
--          NOT what queries inside can do. Queries still run in the
--          caller's auth context. So virtual_key_access couldn't read
--          model_catalog or provider_credential (both required $access = "account").
--
-- FIX:
--   1. Table permissions on model_catalog & provider_credential now allow
--      scoped reads from virtual_key_access via $auth.user (SurrealDB
--      dereferences the record<user> on the virtual_api_key record).
--   2. Field-level permission on encrypted_api_key blocks virtual_key_access.
--   3. Database-scoped BEARER access (backend_service) lets the Rust backend
--      fetch the encrypted key separately through an explicitly permitted
--      function — NOT root, just a scoped credential.
--   4. request_log permissions updated so virtual_key_access can create logs.
--
-- PROXY FLOW:
--   1. Rust receives request with virtual key
--   2. Authenticates to SurrealDB via virtual_key_access (existing mechanism)
--   3. Calls fn::proxy_resolve_route($model_alias) → routing info (NO secret)
--   4. Rust uses backend_service BEARER token → fn::backend_fetch_secret($cred_id)
--   5. Decrypts AES-256-GCM in Rust → forwards to upstream provider
--
-- SECURITY MODEL:
--   - virtual_key → user_id → model_catalog → provider_credential (indirect)
--   - encrypted_api_key is NEVER readable from virtual_key_access scope
--   - Even if someone connects directly to SurrealDB with a stolen virtual key,
--     they get routing metadata only, never the secret
--   - backend_service is database-scoped (not root), holds no user data, and
--     only reads encrypted_api_key through backend-only permissions
--   - BEARER access exposes $access = "backend_service", but does not rely on
--     a record-style $auth object the way TYPE RECORD access does
-- ============================================================


-- ████████████████████████████████████████████████████████████
-- SECTION 1: USER TABLE
-- ████████████████████████████████████████████████████████████
-- (unchanged from original — included for completeness)

DEFINE TABLE OVERWRITE user SCHEMAFULL
    PERMISSIONS
        FOR select, update, delete
            WHERE id = $auth.id AND $access = "account"
        FOR create
            WHERE $auth = NONE;

DEFINE FIELD IF NOT EXISTS name ON TABLE user TYPE string;
DEFINE FIELD IF NOT EXISTS email ON TABLE user TYPE string
    ASSERT string::is_email($value);
DEFINE FIELD IF NOT EXISTS password ON TABLE user TYPE string;
DEFINE FIELD OVERWRITE enabled ON TABLE user TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE user TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE user TYPE datetime VALUE time::now();

DEFINE INDEX IF NOT EXISTS user_email_unique ON TABLE user FIELDS email UNIQUE;


-- ████████████████████████████████████████████████████████████
-- SECTION 2: ACCESS METHODS
-- ████████████████████████████████████████████████████████████

-- 2a. Account access (user signup/signin) — unchanged
DEFINE ACCESS IF NOT EXISTS account ON DATABASE TYPE RECORD
    SIGNUP (
        CREATE user CONTENT {
            name: $name,
            email: $email,
            password: crypto::argon2::generate($password),
            enabled: true
        }
    )
    SIGNIN (
        SELECT * FROM user
        WHERE email = $email
            AND crypto::argon2::compare(password, $password)
            AND enabled = true
    )
    DURATION FOR TOKEN 15m, FOR SESSION 12h;


-- 2b. Virtual key access (proxy authentication) — unchanged
DEFINE ACCESS OVERWRITE virtual_key_access ON DATABASE TYPE RECORD
    SIGNIN (
        SELECT * FROM virtual_api_key
        WHERE key_hash = crypto::sha256($virtual_key)
          AND enabled = true
          AND (expires_at = NONE OR expires_at > time::now())
        LIMIT 1
    )
    DURATION FOR TOKEN 1m, FOR SESSION NONE;


-- 2c. Backend service access — ★ NEW ★
-- Database-scoped BEARER key for the Rust backend.
-- NOT root. Grants system-level access to this database only.
-- The backend BEARER access must be explicitly granted permission to read the
-- encrypted_api_key field and call fn::backend_fetch_secret().
--
-- Usage in Rust:
--   1. On first deploy: run ACCESS GRANT on backend_service to get a bearer key
--   2. Store grant.key in env var VALYMUX_DB_SERVICE_KEY
--   3. Authenticate with: db.signin(record { access: "backend_service", params: { key } })
--   4. Use this connection ONLY for fn::backend_fetch_secret()
--
-- To generate the grant (run once, from a db owner session):
--   ACCESS backend_service ON DATABASE GRANT FOR USER;
--
DEFINE ACCESS OVERWRITE backend_service ON DATABASE TYPE BEARER FOR USER
    DURATION FOR GRANT 365d, FOR TOKEN 1h, FOR SESSION 24h;


-- ████████████████████████████████████████████████████████████
-- SECTION 3: PROVIDER CREDENTIAL TABLE
-- ████████████████████████████████████████████████████████████
-- ★ CHANGED: Added virtual_key_access to SELECT permissions
-- ★ CHANGED: Field-level permission on encrypted_api_key

DEFINE TABLE OVERWRITE provider_credential SCHEMAFULL
    PERMISSIONS
        FOR select
            WHERE (user = $auth.id AND $access = "account")
               -- ★ NEW: virtual_key_access can read credentials owned by their user
               -- $auth.user dereferences the record<user> field on the virtual_api_key record
               OR ($access = "virtual_key_access" AND user = $auth.user)
               -- ★ NEW: backend_service can read encrypted credentials for proxy secret fetch
               OR ($access = "backend_service")
        FOR create, update, delete
            WHERE user = $auth.id AND $access = "account";

DEFINE FIELD IF NOT EXISTS user ON TABLE provider_credential TYPE record<user>;
DEFINE FIELD IF NOT EXISTS provider ON TABLE provider_credential TYPE string;
DEFINE FIELD IF NOT EXISTS label ON TABLE provider_credential TYPE string;

-- ★ CHANGED: Field-level permission blocks virtual_key_access from reading the secret
-- backend_service is explicitly allowed to read this field
DEFINE FIELD OVERWRITE encrypted_api_key ON TABLE provider_credential TYPE string
    PERMISSIONS
        FOR select WHERE $access = "account" AND user = $auth.id
                      OR $access = "backend_service"
        FOR update WHERE $access = "account" AND user = $auth.id
        FOR create FULL;

DEFINE FIELD IF NOT EXISTS tags ON TABLE provider_credential TYPE array;
DEFINE FIELD OVERWRITE enabled ON TABLE provider_credential TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS last_used_at ON TABLE provider_credential TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS sync_status ON TABLE provider_credential TYPE string DEFAULT "pending";
DEFINE FIELD IF NOT EXISTS sync_error ON TABLE provider_credential TYPE option<string>;
DEFINE FIELD IF NOT EXISTS last_synced_at ON TABLE provider_credential TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS model_count ON TABLE provider_credential TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE provider_credential TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE provider_credential TYPE datetime VALUE time::now();

DEFINE INDEX IF NOT EXISTS provider_credential_user_provider_label_unique
    ON TABLE provider_credential FIELDS user, provider, label UNIQUE;


-- ████████████████████████████████████████████████████████████
-- SECTION 4: VIRTUAL API KEY TABLE
-- ████████████████████████████████████████████████████████████
-- (unchanged — permissions already correct)

DEFINE TABLE OVERWRITE virtual_api_key SCHEMAFULL
    PERMISSIONS
        FOR select
            WHERE (user = $auth.id AND $access = "account")
               OR (id = $auth AND $access = "virtual_key_access")
        FOR update, delete
            WHERE user = $auth.id AND $access = "account"
        FOR create
            WHERE user = $auth.id AND $access = "account";

DEFINE FIELD IF NOT EXISTS user ON TABLE virtual_api_key TYPE record<user>;
DEFINE FIELD IF NOT EXISTS name ON TABLE virtual_api_key TYPE string;
DEFINE FIELD IF NOT EXISTS key_prefix ON TABLE virtual_api_key TYPE string;
DEFINE FIELD IF NOT EXISTS key_hash ON TABLE virtual_api_key TYPE string;
DEFINE FIELD IF NOT EXISTS allowed_models ON TABLE virtual_api_key TYPE array;
DEFINE FIELD IF NOT EXISTS tags ON TABLE virtual_api_key TYPE array;
DEFINE FIELD OVERWRITE enabled ON TABLE virtual_api_key TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS expires_at ON TABLE virtual_api_key TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS last_used_at ON TABLE virtual_api_key TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE virtual_api_key TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE virtual_api_key TYPE datetime VALUE time::now();

DEFINE INDEX IF NOT EXISTS virtual_api_key_hash_unique ON TABLE virtual_api_key FIELDS key_hash UNIQUE;


-- ████████████████████████████████████████████████████████████
-- SECTION 5: MODEL CATALOG TABLE
-- ████████████████████████████████████████████████████████████
-- ★ CHANGED: Added virtual_key_access to SELECT permissions

DEFINE TABLE OVERWRITE model_catalog SCHEMAFULL
    PERMISSIONS
        FOR select
            WHERE (user = $auth.id AND $access = "account")
               -- ★ NEW: virtual_key_access can read catalog entries owned by their user
               OR ($access = "virtual_key_access" AND user = $auth.user)
        FOR create, update, delete
            WHERE user = $auth.id AND $access = "account";

DEFINE FIELD IF NOT EXISTS user ON TABLE model_catalog TYPE record<user>;
DEFINE FIELD IF NOT EXISTS provider_credential ON TABLE model_catalog TYPE record<provider_credential>;
DEFINE FIELD IF NOT EXISTS alias ON TABLE model_catalog TYPE string;
DEFINE FIELD IF NOT EXISTS provider ON TABLE model_catalog TYPE string;
DEFINE FIELD IF NOT EXISTS upstream_model ON TABLE model_catalog TYPE string;
DEFINE FIELD IF NOT EXISTS display_name ON TABLE model_catalog TYPE string;
DEFINE FIELD IF NOT EXISTS description ON TABLE model_catalog TYPE option<string>;
DEFINE FIELD IF NOT EXISTS context_window_tokens ON TABLE model_catalog TYPE int;
DEFINE FIELD IF NOT EXISTS max_output_tokens ON TABLE model_catalog TYPE int;
DEFINE FIELD IF NOT EXISTS supports_streaming ON TABLE model_catalog TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS supports_thinking ON TABLE model_catalog TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS thinking_required ON TABLE model_catalog TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS supports_temperature ON TABLE model_catalog TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS temperature_fixed_to ON TABLE model_catalog TYPE option<float>;
DEFINE FIELD IF NOT EXISTS temperature_min ON TABLE model_catalog TYPE option<float>;
DEFINE FIELD IF NOT EXISTS temperature_max ON TABLE model_catalog TYPE option<float>;
DEFINE FIELD IF NOT EXISTS supports_top_p ON TABLE model_catalog TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS supports_system_messages ON TABLE model_catalog TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS supports_tools ON TABLE model_catalog TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS supports_vision ON TABLE model_catalog TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS supports_json_mode ON TABLE model_catalog TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS supports_parallel_tool_calls ON TABLE model_catalog TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS enabled ON TABLE model_catalog TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE model_catalog TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE model_catalog TYPE datetime VALUE time::now();

DEFINE INDEX IF NOT EXISTS model_catalog_user_alias_unique
    ON TABLE model_catalog FIELDS user, alias UNIQUE;


-- ████████████████████████████████████████████████████████████
-- SECTION 6: REQUEST LOG TABLE
-- ████████████████████████████████████████████████████████████
-- ★ CHANGED: virtual_key_access can now CREATE log entries

DEFINE TABLE OVERWRITE request_log SCHEMAFULL
    PERMISSIONS
        FOR select
            WHERE user = $auth.id AND $access = "account"
        FOR create
            WHERE ($access = "account" AND user = $auth.id)
               -- ★ NEW: virtual_key_access can create log entries (scoped to their user)
               OR ($access = "virtual_key_access" AND user = $auth.user)
        FOR update, delete
            NONE;

DEFINE FIELD IF NOT EXISTS request_id ON TABLE request_log TYPE string;
DEFINE FIELD IF NOT EXISTS user ON TABLE request_log TYPE record<user>;
DEFINE FIELD IF NOT EXISTS virtual_api_key ON TABLE request_log TYPE option<record<virtual_api_key>>;
-- ★ REMOVED: provider_credential field from request_log
-- Logging which provider credential was used creates a traceable link.
-- Instead, log only the provider name (which is already public info).
DEFINE FIELD IF NOT EXISTS model_alias ON TABLE request_log TYPE string;
DEFINE FIELD IF NOT EXISTS provider ON TABLE request_log TYPE string;
DEFINE FIELD IF NOT EXISTS upstream_model ON TABLE request_log TYPE string;
DEFINE FIELD IF NOT EXISTS status_code ON TABLE request_log TYPE int;
DEFINE FIELD IF NOT EXISTS latency_ms ON TABLE request_log TYPE int;
DEFINE FIELD IF NOT EXISTS stream ON TABLE request_log TYPE bool;
DEFINE FIELD IF NOT EXISTS request_url ON TABLE request_log TYPE string;
DEFINE FIELD IF NOT EXISTS error_message ON TABLE request_log TYPE option<string>;
DEFINE FIELD IF NOT EXISTS usage_input_tokens ON TABLE request_log TYPE option<int>;
DEFINE FIELD IF NOT EXISTS usage_output_tokens ON TABLE request_log TYPE option<int>;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE request_log TYPE datetime VALUE time::now() READONLY;

DEFINE INDEX IF NOT EXISTS request_log_request_id_unique ON TABLE request_log FIELDS request_id UNIQUE;


-- ████████████████████████████████████████████████████████████
-- SECTION 7: USER FUNCTIONS (account scope)
-- ████████████████████████████████████████████████████████████
-- (unchanged)

DEFINE FUNCTION OVERWRITE fn::get_current_user() {
    LET $user = (SELECT * FROM $auth.id)[0];
    IF $user = NONE {
        THROW "authenticated user not found";
    };
    IF $user.enabled = false {
        THROW "account is disabled";
    };
    RETURN $user;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::update_profile($changed_name: string) {
    UPDATE user SET name = $changed_name WHERE id = $auth.id;
    RETURN (SELECT id, * FROM user WHERE id = $auth.id)[0];
} PERMISSIONS WHERE $auth != NONE AND $access = "account";


-- ████████████████████████████████████████████████████████████
-- SECTION 8: PROVIDER CREDENTIAL FUNCTIONS (account scope)
-- ████████████████████████████████████████████████████████████
-- (unchanged — these only run in account scope)

DEFINE FUNCTION OVERWRITE fn::list_provider_credentials() {
    RETURN SELECT * FROM provider_credential
           WHERE user = $auth.id
           ORDER BY created_at ASC;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::get_provider_credential($id: record<provider_credential>) {
    RETURN (SELECT * FROM $id WHERE user = $auth.id)[0];
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::create_provider_credential(
    $provider: string,
    $label: string,
    $encrypted_api_key: string,
    $tags: array
) {
    RETURN CREATE provider_credential CONTENT {
        user: $auth.id,
        provider: $provider,
        label: $label,
        encrypted_api_key: $encrypted_api_key,
        tags: $tags,
        enabled: true,
        sync_status: "pending",
        model_count: 0
    };
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::update_provider_credential(
    $id: record<provider_credential>,
    $label: string,
    $tags: array,
    $enabled: bool,
    $encrypted_api_key: option<string>
) {
    LET $existing = (SELECT * FROM $id WHERE user = $auth.id)[0];
    IF $existing = NONE {
        THROW "provider credential not found or access denied";
    };

    IF $encrypted_api_key != NONE {
        LET $updated = (UPDATE $id MERGE {
            label: $label,
            tags: $tags,
            enabled: $enabled,
            encrypted_api_key: $encrypted_api_key,
            sync_status: "pending"
        } WHERE user = $auth.id RETURN AFTER)[0];
        RETURN $updated;
    };

    RETURN (UPDATE $id MERGE {
        label: $label,
        tags: $tags,
        enabled: $enabled
    } WHERE user = $auth.id RETURN AFTER)[0];
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::delete_provider_credential($id: record<provider_credential>) {
    LET $before = (SELECT * FROM $id WHERE user = $auth.id)[0];
    IF $before = NONE {
        RETURN NONE;
    };

    DELETE model_catalog WHERE provider_credential = $id AND user = $auth.id;
    DELETE $id WHERE user = $auth.id;

    RETURN $before;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";


-- ████████████████████████████████████████████████████████████
-- SECTION 9: VIRTUAL API KEY FUNCTIONS (account scope)
-- ████████████████████████████████████████████████████████████
-- (unchanged)

DEFINE FUNCTION OVERWRITE fn::list_virtual_api_keys() {
    RETURN SELECT * FROM virtual_api_key
           WHERE user = $auth.id
           ORDER BY created_at ASC;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::get_virtual_api_key($id: record<virtual_api_key>) {
    RETURN (SELECT * FROM $id WHERE user = $auth.id)[0];
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::validate_model_aliases($aliases: array) {
    IF array::len($aliases) = 0 {
        RETURN true;
    };

    LET $existing = (
        SELECT VALUE alias FROM model_catalog
        WHERE user = $auth.id
          AND alias INSIDE $aliases
          AND enabled = true
          AND provider_credential.enabled = true
    );

    LET $missing = array::complement($aliases, $existing);

    IF array::len($missing) > 0 {
        THROW string::concat(
            "model alias(es) not found in your catalog: ",
            string::join(", ", $missing)
        );
    };

    RETURN true;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::create_virtual_api_key(
    $name: string,
    $key_prefix: string,
    $key_hash: string,
    $allowed_models: array,
    $tags: array,
    $expires_at: option<datetime>
) {
    IF array::len($allowed_models) > 0 {
        LET $existing_aliases = (
            SELECT VALUE alias FROM model_catalog
            WHERE user = $auth.id
              AND alias INSIDE $allowed_models
              AND enabled = true
              AND provider_credential.enabled = true
        );
        LET $missing = array::complement($allowed_models, $existing_aliases);
        IF array::len($missing) > 0 {
            THROW string::concat(
                "model alias(es) not found in your catalog: ",
                string::join(", ", $missing)
            );
        };
    };

    RETURN CREATE virtual_api_key CONTENT {
        user: $auth.id,
        name: $name,
        key_prefix: $key_prefix,
        key_hash: $key_hash,
        allowed_models: $allowed_models,
        tags: $tags,
        enabled: true,
        expires_at: $expires_at
    };
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::update_virtual_api_key(
    $id: record<virtual_api_key>,
    $name: string,
    $allowed_models: array,
    $tags: array,
    $enabled: bool,
    $expires_at: option<datetime>
) {
    LET $existing = (SELECT * FROM $id WHERE user = $auth.id)[0];
    IF $existing = NONE {
        THROW "virtual API key not found or access denied";
    };

    IF array::len($allowed_models) > 0 {
        LET $existing_aliases = (
            SELECT VALUE alias FROM model_catalog
            WHERE user = $auth.id
              AND alias INSIDE $allowed_models
              AND enabled = true
              AND provider_credential.enabled = true
        );
        LET $missing = array::complement($allowed_models, $existing_aliases);
        IF array::len($missing) > 0 {
            THROW string::concat(
                "model alias(es) not found in your catalog: ",
                string::join(", ", $missing)
            );
        };
    };

    RETURN (UPDATE $id MERGE {
        name: $name,
        allowed_models: $allowed_models,
        tags: $tags,
        enabled: $enabled,
        expires_at: $expires_at
    } WHERE user = $auth.id RETURN AFTER)[0];
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::delete_virtual_api_key($id: record<virtual_api_key>) {
    LET $before = (SELECT * FROM $id WHERE user = $auth.id)[0];
    IF $before = NONE {
        RETURN NONE;
    };

    DELETE $id WHERE user = $auth.id;

    RETURN $before;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";


-- ████████████████████████████████████████████████████████████
-- SECTION 10: MODEL CATALOG FUNCTIONS (account scope)
-- ████████████████████████████████████████████████████████████
-- (unchanged)

DEFINE FUNCTION OVERWRITE fn::list_usable_models() {
    RETURN SELECT * FROM model_catalog
           WHERE user = $auth.id
             AND enabled = true
             AND provider_credential.enabled = true
           ORDER BY alias ASC;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::get_model_by_alias($alias: string) {
    RETURN (
        SELECT * FROM model_catalog
        WHERE user = $auth.id
          AND alias = $alias
          AND enabled = true
        LIMIT 1
    )[0];
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::list_models_for_credential($cred_id: record<provider_credential>) {
    LET $cred = (SELECT * FROM $cred_id WHERE user = $auth.id)[0];
    IF $cred = NONE {
        THROW "credential not found or access denied";
    };

    RETURN SELECT * FROM model_catalog
           WHERE provider_credential = $cred_id
             AND user = $auth.id
             AND enabled = true
           ORDER BY alias ASC;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::sync_models(
    $cred_id: record<provider_credential>,
    $models: array
) {
    LET $cred = (SELECT * FROM $cred_id WHERE user = $auth.id)[0];
    IF $cred = NONE {
        THROW "credential not found or access denied";
    };

    DELETE model_catalog WHERE provider_credential = $cred_id AND user = $auth.id;

    FOR $model IN $models {
        CREATE model_catalog CONTENT {
            user: $auth.id,
            provider_credential: $cred_id,
            alias: $model.alias,
            provider: $model.provider,
            upstream_model: $model.upstream_model,
            display_name: $model.display_name,
            description: $model.description,
            context_window_tokens: $model.context_window_tokens,
            max_output_tokens: $model.max_output_tokens,
            supports_streaming: $model.supports_streaming,
            supports_thinking: $model.supports_thinking,
            thinking_required: $model.thinking_required,
            supports_temperature: $model.supports_temperature,
            temperature_fixed_to: $model.temperature_fixed_to,
            temperature_min: $model.temperature_min,
            temperature_max: $model.temperature_max,
            supports_top_p: $model.supports_top_p,
            supports_system_messages: $model.supports_system_messages,
            supports_tools: $model.supports_tools,
            supports_vision: $model.supports_vision,
            supports_json_mode: $model.supports_json_mode,
            supports_parallel_tool_calls: $model.supports_parallel_tool_calls,
            enabled: true
        };
    };

    LET $count = array::len($models);

    UPDATE $cred_id MERGE {
        model_count: $count,
        last_synced_at: time::now(),
        sync_status: "synced",
        sync_error: NONE
    } WHERE user = $auth.id;

    RETURN $count;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::set_credential_sync_status(
    $cred_id: record<provider_credential>,
    $status: string,
    $error: option<string>
) {
    LET $cred = (SELECT * FROM $cred_id WHERE user = $auth.id)[0];
    IF $cred = NONE {
        THROW "credential not found or access denied";
    };

    UPDATE $cred_id MERGE {
        sync_status: $status,
        sync_error: $error
    } WHERE user = $auth.id;

    RETURN true;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";


-- ████████████████████████████████████████████████████████████
-- SECTION 11: PROXY FUNCTIONS (data plane)
-- ████████████████████████████████████████████████████████████
-- ★ CHANGED: proxy_resolve_route no longer returns encrypted_api_key
-- ★ NEW: backend_fetch_secret for the Rust backend to get the key

-- ──────────────────────────────────────────────────────────
-- fn::proxy_resolve_route
-- Called from virtual_key_access scope.
-- Returns routing metadata WITHOUT the provider secret.
-- ──────────────────────────────────────────────────────────
DEFINE FUNCTION OVERWRITE fn::proxy_resolve_route($model_alias: string) {
    -- Gate: only callable from virtual_key_access
    IF $access != "virtual_key_access" {
        THROW "proxy route resolution requires virtual key access";
    };

    -- $auth IS the virtual_api_key record in this scope
    -- This SELECT works because virtual_api_key permissions allow:
    --   id = $auth AND $access = "virtual_key_access"
    LET $key = (SELECT * FROM $auth)[0];
    IF $key = NONE {
        THROW "invalid or expired virtual API key";
    };

    -- Check allowed_models whitelist
    IF array::len($key.allowed_models) > 0 AND !($model_alias INSIDE $key.allowed_models) {
        THROW "virtual API key is not allowed to use this model";
    };

    -- Resolve model from catalog
    -- ★ THIS NOW WORKS because model_catalog permissions allow:
    --   $access = "virtual_key_access" AND user = $auth.user
    LET $model = (
        SELECT * FROM model_catalog
        WHERE user = $key.user
          AND alias = $model_alias
          AND enabled = true
        LIMIT 1
    )[0];

    IF $model = NONE {
        THROW "requested model not found in catalog";
    };

    -- Resolve provider credential (metadata only, encrypted_api_key is field-blocked)
    -- ★ THIS NOW WORKS because provider_credential permissions allow:
    --   $access = "virtual_key_access" AND user = $auth.user
    -- ★ encrypted_api_key field returns NONE due to field-level permission
    LET $cred = (
        SELECT * FROM $model.provider_credential
        WHERE enabled = true
    )[0];

    IF $cred = NONE {
        THROW "provider credential is disabled or not found";
    };

    -- Update last_used_at on the virtual key
    UPDATE $auth SET last_used_at = time::now();

    -- Return routing info — NO encrypted_api_key in here
    RETURN {
        virtual_key_id: $key.id,
        user_id: $key.user,
        model: {
            alias: $model.alias,
            provider: $model.provider,
            upstream_model: $model.upstream_model,
            context_window_tokens: $model.context_window_tokens,
            max_output_tokens: $model.max_output_tokens,
            supports_streaming: $model.supports_streaming,
            supports_thinking: $model.supports_thinking,
            thinking_required: $model.thinking_required,
            supports_temperature: $model.supports_temperature,
            temperature_fixed_to: $model.temperature_fixed_to,
            temperature_min: $model.temperature_min,
            temperature_max: $model.temperature_max,
            supports_top_p: $model.supports_top_p,
            supports_system_messages: $model.supports_system_messages,
            supports_tools: $model.supports_tools,
            supports_vision: $model.supports_vision,
            supports_json_mode: $model.supports_json_mode,
            supports_parallel_tool_calls: $model.supports_parallel_tool_calls
        },
        provider_credential_id: $cred.id,
        provider: $cred.provider
    };
} PERMISSIONS FULL;

-- ──────────────────────────────────────────────────────────
-- fn::backend_fetch_secret — ★ NEW ★
-- Called from backend_service BEARER scope ONLY.
-- Returns the encrypted_api_key for a given credential ID.
-- backend_service is explicitly permitted to read encrypted_api_key.
-- ──────────────────────────────────────────────────────────
DEFINE FUNCTION OVERWRITE fn::backend_fetch_secret(
    $cred_id: record<provider_credential>
) {
    LET $cred = (SELECT encrypted_api_key FROM $cred_id WHERE enabled = true)[0];
    IF $cred = NONE {
        THROW "provider credential not found or disabled";
    };
    RETURN $cred.encrypted_api_key;
} PERMISSIONS WHERE $access = "backend_service";

-- ──────────────────────────────────────────────────────────
-- fn::proxy_log_request — ★ UPDATED ★
-- Removed provider_credential_id from log entries.
-- ──────────────────────────────────────────────────────────
DEFINE FUNCTION OVERWRITE fn::proxy_log_request(
    $request_id: string,
    $model_alias: string,
    $provider: string,
    $upstream_model: string,
    $status_code: int,
    $latency_ms: int,
    $stream: bool,
    $request_url: string,
    $error_message: option<string>,
    $usage_input_tokens: option<int>,
    $usage_output_tokens: option<int>
) {
    IF $access != "virtual_key_access" {
        THROW "request logging requires virtual key access";
    };

    LET $key = (SELECT * FROM $auth)[0];
    IF $key = NONE {
        THROW "invalid or expired virtual API key";
    };

    -- ★ CHANGED: no provider_credential in the log
    RETURN CREATE request_log CONTENT {
        request_id: $request_id,
        user: $key.user,
        virtual_api_key: $key.id,
        model_alias: $model_alias,
        provider: $provider,
        upstream_model: $upstream_model,
        status_code: $status_code,
        latency_ms: $latency_ms,
        stream: $stream,
        request_url: $request_url,
        error_message: $error_message,
        usage_input_tokens: $usage_input_tokens,
        usage_output_tokens: $usage_output_tokens
    };
} PERMISSIONS FULL;
