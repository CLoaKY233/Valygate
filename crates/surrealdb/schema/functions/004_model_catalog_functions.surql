-- Model Catalog Functions
-- PERMISSIONS WHERE $auth != NONE: $auth.id IS available inside the function body

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
