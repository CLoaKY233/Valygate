DEFINE TABLE IF NOT EXISTS provider_credential SCHEMAFULL
    PERMISSIONS
        FOR select, update, delete
            WHERE user = $auth.id
        FOR create
            WHERE user = $auth.id;

DEFINE FIELD IF NOT EXISTS user ON TABLE provider_credential TYPE record<user>;
DEFINE FIELD IF NOT EXISTS provider ON TABLE provider_credential TYPE string;
DEFINE FIELD IF NOT EXISTS label ON TABLE provider_credential TYPE string;
DEFINE FIELD IF NOT EXISTS encrypted_api_key ON TABLE provider_credential TYPE string;
DEFINE FIELD IF NOT EXISTS tags ON TABLE provider_credential TYPE array;
DEFINE FIELD IF NOT EXISTS enabled ON TABLE provider_credential TYPE bool VALUE true;
DEFINE FIELD IF NOT EXISTS last_used_at ON TABLE provider_credential TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE provider_credential TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE provider_credential TYPE datetime VALUE time::now();

DEFINE INDEX IF NOT EXISTS provider_credential_user_provider_label_unique
    ON TABLE provider_credential FIELDS user, provider, label UNIQUE;
