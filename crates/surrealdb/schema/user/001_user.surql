DEFINE TABLE IF NOT EXISTS user SCHEMAFULL
    PERMISSIONS
        FOR select, update, delete
            WHERE id = $auth.id
        FOR create
            WHERE $auth = NONE;

DEFINE FIELD IF NOT EXISTS name ON TABLE user TYPE string;
DEFINE FIELD IF NOT EXISTS email ON TABLE user TYPE string ASSERT string::is_email($value);
DEFINE FIELD IF NOT EXISTS password ON TABLE user TYPE string;
DEFINE FIELD IF NOT EXISTS enabled ON TABLE user TYPE bool VALUE true;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE user TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE user TYPE datetime VALUE time::now();

DEFINE INDEX IF NOT EXISTS user_email_unique ON TABLE user FIELDS email UNIQUE;
