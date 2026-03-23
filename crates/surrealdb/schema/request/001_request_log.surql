DEFINE TABLE IF NOT EXISTS request_log SCHEMAFULL
    PERMISSIONS
        FOR select
            WHERE user = $auth.id
        FOR create
            NONE
        FOR update, delete
            NONE;

DEFINE FIELD IF NOT EXISTS request_id ON TABLE request_log TYPE string;
DEFINE FIELD IF NOT EXISTS user ON TABLE request_log TYPE record<user>;
DEFINE FIELD IF NOT EXISTS virtual_api_key ON TABLE request_log TYPE option<record<virtual_api_key>>;
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
