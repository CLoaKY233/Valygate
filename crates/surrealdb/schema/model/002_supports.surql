DEFINE TABLE OVERWRITE supports TYPE RELATION IN provider_credential OUT model_definition ENFORCED
    SCHEMAFULL
    PERMISSIONS
        FOR select
            WHERE ($access = "account" AND in.user = $auth.id)
               OR ($access = "virtual_key_access" AND in.user = $auth.user)
        FOR create, update, delete
            WHERE $access = "account" AND in.user = $auth.id;

DEFINE FIELD OVERWRITE in ON TABLE supports TYPE record<provider_credential>;
DEFINE FIELD OVERWRITE out ON TABLE supports TYPE record<model_definition>;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE supports TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE supports TYPE datetime VALUE time::now();

DEFINE INDEX IF NOT EXISTS supports_unique_edge
    ON TABLE supports FIELDS in, out UNIQUE;

-- Performance indices: traversal from credential → models and reverse
DEFINE INDEX IF NOT EXISTS idx_supports_in  ON TABLE supports FIELDS in;
DEFINE INDEX IF NOT EXISTS idx_supports_out ON TABLE supports FIELDS out;
