DEFINE TABLE OVERWRITE virtual_key_route TYPE RELATION IN virtual_api_key OUT provider_credential ENFORCED
    SCHEMAFULL
    PERMISSIONS
        FOR select
            WHERE ($access = "account" AND in.user = $auth.id)
               OR ($access = "virtual_key_access" AND in = $auth)
        FOR create, update, delete
            WHERE $access = "account" AND in.user = $auth.id;

DEFINE FIELD OVERWRITE in ON TABLE virtual_key_route TYPE record<virtual_api_key>;
DEFINE FIELD OVERWRITE out ON TABLE virtual_key_route TYPE record<provider_credential>;
DEFINE FIELD IF NOT EXISTS model_alias ON TABLE virtual_key_route TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE virtual_key_route TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE virtual_key_route TYPE datetime VALUE time::now();

DEFINE INDEX IF NOT EXISTS virtual_key_route_alias_unique
    ON TABLE virtual_key_route FIELDS in, model_alias UNIQUE;
