DEFINE TABLE OVERWRITE provider_credential SCHEMAFULL
    PERMISSIONS
        FOR select
            WHERE (user = $auth.id AND $access = "account")
               OR ($access = "virtual_key_access" AND user = $auth.user)
               OR ($access = "backend_service")
        FOR update, delete
            WHERE user = $auth.id AND $access = "account"
        FOR create
            WHERE user = $auth.id AND $access = "account";

DEFINE FIELD IF NOT EXISTS user ON TABLE provider_credential TYPE record<user>;
DEFINE FIELD OVERWRITE provider ON TABLE provider_credential TYPE string
    ASSERT string::len($value) >= 1 AND string::len($value) <= 64;
DEFINE FIELD OVERWRITE label ON TABLE provider_credential TYPE string
    ASSERT string::len($value) >= 1 AND string::len($value) <= 256;
DEFINE FIELD OVERWRITE encrypted_api_key ON TABLE provider_credential TYPE string
    PERMISSIONS
        FOR select WHERE $access = "account" AND user = $auth.id
                      OR $access = "backend_service"
        FOR update WHERE $access = "account" AND user = $auth.id
        FOR create FULL;
DEFINE FIELD IF NOT EXISTS tags ON TABLE provider_credential TYPE array;
DEFINE FIELD OVERWRITE enabled ON TABLE provider_credential TYPE bool DEFAULT true;
DEFINE FIELD IF NOT EXISTS last_used_at ON TABLE provider_credential TYPE option<datetime>;
DEFINE FIELD OVERWRITE sync_status ON TABLE provider_credential TYPE string DEFAULT "pending"
    ASSERT $value IN ["pending", "syncing", "synced", "completed", "failed", "deleting"];
DEFINE FIELD IF NOT EXISTS sync_started_at ON TABLE provider_credential TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS sync_error ON TABLE provider_credential TYPE option<string>;
DEFINE FIELD IF NOT EXISTS last_synced_at ON TABLE provider_credential TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS model_count ON TABLE provider_credential TYPE int DEFAULT 0;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE provider_credential TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE provider_credential TYPE datetime VALUE time::now();

DEFINE INDEX IF NOT EXISTS provider_credential_user_provider_label_unique
    ON TABLE provider_credential FIELDS user, provider, label UNIQUE;

-- Performance index: credential lookups by owner (list, cascade delete, model sync)
DEFINE INDEX IF NOT EXISTS idx_provider_credential_user
    ON TABLE provider_credential FIELDS user;
