DEFINE TABLE IF NOT EXISTS model_catalog SCHEMAFULL
    PERMISSIONS
        FOR select
            WHERE $auth != NONE
        FOR create, update, delete
            NONE;

DEFINE FIELD IF NOT EXISTS alias ON TABLE model_catalog TYPE string;
DEFINE FIELD IF NOT EXISTS display_name ON TABLE model_catalog TYPE string;
DEFINE FIELD IF NOT EXISTS provider ON TABLE model_catalog TYPE string;
DEFINE FIELD IF NOT EXISTS upstream_model ON TABLE model_catalog TYPE string;
DEFINE FIELD IF NOT EXISTS description ON TABLE model_catalog TYPE string;
DEFINE FIELD IF NOT EXISTS tags ON TABLE model_catalog TYPE array;
DEFINE FIELD IF NOT EXISTS enabled ON TABLE model_catalog TYPE bool VALUE true;
DEFINE FIELD IF NOT EXISTS context_window_tokens ON TABLE model_catalog TYPE int;
DEFINE FIELD IF NOT EXISTS max_output_tokens ON TABLE model_catalog TYPE int;
DEFINE FIELD IF NOT EXISTS supports_streaming ON TABLE model_catalog TYPE bool VALUE true;
DEFINE FIELD IF NOT EXISTS supports_thinking ON TABLE model_catalog TYPE bool VALUE false;
DEFINE FIELD IF NOT EXISTS thinking_required ON TABLE model_catalog TYPE bool VALUE false;
DEFINE FIELD IF NOT EXISTS supports_temperature ON TABLE model_catalog TYPE bool VALUE true;
DEFINE FIELD IF NOT EXISTS temperature_fixed_to ON TABLE model_catalog TYPE option<number>;
DEFINE FIELD IF NOT EXISTS temperature_min ON TABLE model_catalog TYPE option<number>;
DEFINE FIELD IF NOT EXISTS temperature_max ON TABLE model_catalog TYPE option<number>;
DEFINE FIELD IF NOT EXISTS supports_top_p ON TABLE model_catalog TYPE bool VALUE true;
DEFINE FIELD IF NOT EXISTS supports_system_messages ON TABLE model_catalog TYPE bool VALUE true;
DEFINE FIELD IF NOT EXISTS supports_tools ON TABLE model_catalog TYPE bool VALUE false;
DEFINE FIELD IF NOT EXISTS supports_vision ON TABLE model_catalog TYPE bool VALUE false;
DEFINE FIELD IF NOT EXISTS supports_json_mode ON TABLE model_catalog TYPE bool VALUE false;
DEFINE FIELD IF NOT EXISTS supports_parallel_tool_calls ON TABLE model_catalog TYPE bool VALUE false;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE model_catalog TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE model_catalog TYPE datetime VALUE time::now();

DEFINE INDEX IF NOT EXISTS model_catalog_alias_unique ON TABLE model_catalog FIELDS alias UNIQUE;
