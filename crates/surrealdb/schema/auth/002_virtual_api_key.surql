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
