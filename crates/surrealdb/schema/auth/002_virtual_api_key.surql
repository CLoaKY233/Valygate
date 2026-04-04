DEFINE TABLE OVERWRITE virtual_api_key SCHEMAFULL
    PERMISSIONS
        FOR select
            WHERE (user = $auth.id AND $access = "account")
               OR (id = $auth AND $access = "virtual_key_access")
        -- account: full update; virtual_key_access: only its own record (field-level perms restrict what)
        FOR update
            WHERE (user = $auth.id AND $access = "account")
               OR (id = $auth AND $access = "virtual_key_access")
        FOR delete
            WHERE user = $auth.id AND $access = "account"
        FOR create
            WHERE user = $auth.id AND $access = "account";

-- Sensitive fields: only the owning account session may update these.
DEFINE FIELD OVERWRITE user ON TABLE virtual_api_key TYPE record<user>
    PERMISSIONS FOR update WHERE user = $auth.id AND $access = "account";
DEFINE FIELD OVERWRITE name ON TABLE virtual_api_key TYPE string
    PERMISSIONS FOR update WHERE user = $auth.id AND $access = "account";
DEFINE FIELD OVERWRITE key_prefix ON TABLE virtual_api_key TYPE string
    PERMISSIONS FOR update WHERE user = $auth.id AND $access = "account";
DEFINE FIELD OVERWRITE key_hash ON TABLE virtual_api_key TYPE string
    PERMISSIONS FOR update WHERE user = $auth.id AND $access = "account";
DEFINE FIELD OVERWRITE allowed_models ON TABLE virtual_api_key TYPE array
    PERMISSIONS FOR update WHERE user = $auth.id AND $access = "account";
DEFINE FIELD OVERWRITE tags ON TABLE virtual_api_key TYPE array
    PERMISSIONS FOR update WHERE user = $auth.id AND $access = "account";
DEFINE FIELD OVERWRITE enabled ON TABLE virtual_api_key TYPE bool DEFAULT true
    PERMISSIONS FOR update WHERE user = $auth.id AND $access = "account";
DEFINE FIELD OVERWRITE expires_at ON TABLE virtual_api_key TYPE option<datetime>
    PERMISSIONS FOR update WHERE user = $auth.id AND $access = "account";
-- last_used_at: account can update it manually; virtual_key_access updates it on each proxy call.
DEFINE FIELD OVERWRITE last_used_at ON TABLE virtual_api_key TYPE option<datetime>
    PERMISSIONS
        FOR select WHERE (user = $auth.id AND $access = "account") OR (id = $auth AND $access = "virtual_key_access")
        FOR update WHERE (user = $auth.id AND $access = "account") OR (id = $auth AND $access = "virtual_key_access");
DEFINE FIELD OVERWRITE created_at ON TABLE virtual_api_key TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD OVERWRITE updated_at ON TABLE virtual_api_key TYPE datetime VALUE time::now();

DEFINE INDEX IF NOT EXISTS virtual_api_key_hash_unique ON TABLE virtual_api_key FIELDS key_hash UNIQUE;

-- Performance index: key lookups by owner (list, cascade delete, usage queries)
DEFINE INDEX IF NOT EXISTS idx_virtual_api_key_user ON TABLE virtual_api_key FIELDS user;
