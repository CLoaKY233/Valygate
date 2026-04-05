-- Model Definition and Credential Support Functions

DEFINE FUNCTION OVERWRITE fn::list_usable_models() {
    LET $credential_ids = (
        SELECT VALUE id FROM provider_credential
        WHERE user = $auth.id
          AND enabled = true
    );

    LET $model_ids = (
        SELECT VALUE out FROM supports
        WHERE in INSIDE $credential_ids
    );

    RETURN SELECT * FROM model_definition
           WHERE id INSIDE $model_ids
           ORDER BY alias ASC;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::get_model_by_alias($alias: string) {
    LET $credential_ids = (
        SELECT VALUE id FROM provider_credential
        WHERE user = $auth.id
          AND enabled = true
    );

    LET $model_ids = (
        SELECT VALUE out FROM supports
        WHERE in INSIDE $credential_ids
    );

    RETURN (
        SELECT * FROM model_definition
        WHERE alias = $alias
          AND id INSIDE $model_ids
        LIMIT 1
    )[0];
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::list_models_for_credential($cred_id: record<provider_credential>) {
    LET $cred = (SELECT * FROM $cred_id WHERE user = $auth.id)[0];
    IF $cred = NONE {
        THROW "credential not found or access denied";
    };

    LET $model_ids = (
        SELECT VALUE out FROM supports
        WHERE in = $cred_id
    );

    RETURN SELECT * FROM model_definition
           WHERE id INSIDE $model_ids
           ORDER BY alias ASC;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::sync_models(
    $cred_id: record<provider_credential>,
    $models: array
) {
    LET $cred = (SELECT * FROM $cred_id WHERE user = $auth.id)[0];
    IF $cred = NONE {
        RETURN 0;
    };

    IF $cred.sync_status = "deleting" {
        RETURN 0;
    };

    DELETE supports WHERE in = $cred_id;

    FOR $model IN $models {
        LET $existing_model = (
            SELECT * FROM model_definition
            WHERE alias = $model.alias
            LIMIT 1
        )[0];

        LET $model_id = IF $existing_model = NONE {
            (
                CREATE model_definition CONTENT {
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
                    supports_parallel_tool_calls: $model.supports_parallel_tool_calls
                } RETURN AFTER
            )[0].id
        } ELSE {
            (UPDATE $existing_model.id MERGE {
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
                updated_at: time::now()
            } RETURN AFTER)[0].id
        };

        RELATE ONLY $cred_id->supports->$model_id;
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
        RETURN false;
    };

    IF $cred.sync_status = "deleting" {
        RETURN false;
    };

    UPDATE $cred_id MERGE {
        sync_status: $status,
        sync_error: $error,
        sync_started_at: IF $status = "syncing" THEN sync_started_at ELSE NONE END
    } WHERE user = $auth.id;

    RETURN true;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";
