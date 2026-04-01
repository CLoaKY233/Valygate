DEFINE FUNCTION OVERWRITE fn::proxy_resolve_route($model_alias: string) {
    IF $access != "virtual_key_access" {
        THROW "proxy route resolution requires virtual key access";
    };

    LET $key = (SELECT * FROM $auth)[0];
    IF $key = NONE {
        THROW "invalid or expired virtual API key";
    };

    IF array::len($key.allowed_models) > 0 AND !($model_alias INSIDE $key.allowed_models) {
        THROW "virtual API key is not allowed to use this model";
    };

    LET $model = (
        SELECT id, * FROM model_catalog
        WHERE user = $key.user
          AND alias = $model_alias
          AND enabled = true
        LIMIT 1
    )[0];

    IF $model = NONE {
        THROW "requested model not found in catalog";
    };

    LET $cred = (
        SELECT * FROM $model.provider_credential
        WHERE enabled = true
    )[0];
    IF $cred = NONE {
        THROW "provider credential is disabled or not found";
    };

    UPDATE $auth SET last_used_at = time::now();

    RETURN {
        virtual_key_id: $key.id,
        user_id: $key.user,
        model: {
            alias: $model.alias,
            provider: $model.provider,
            upstream_model: $model.upstream_model,
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
        },
        provider_credential_id: $cred.id,
        provider: $cred.provider
    };
} PERMISSIONS FULL;

DEFINE FUNCTION OVERWRITE fn::backend_fetch_secret(
    $cred_id: record<provider_credential>
) {
    LET $cred = (SELECT encrypted_api_key FROM $cred_id WHERE enabled = true)[0];
    IF $cred = NONE {
        THROW "provider credential not found or disabled";
    };
    RETURN $cred.encrypted_api_key;
} PERMISSIONS WHERE $access = "backend_service";

DEFINE FUNCTION OVERWRITE fn::proxy_log_request(
    $request_id: string,
    $model_alias: string,
    $provider: string,
    $upstream_model: string,
    $status_code: int,
    $latency_ms: int,
    $stream: bool,
    $request_url: string,
    $error_message: option<string>,
    $usage_input_tokens: option<int>,
    $usage_output_tokens: option<int>
) {
    IF $access != "virtual_key_access" {
        THROW "request logging requires virtual key access";
    };

    LET $key = (SELECT * FROM $auth)[0];
    IF $key = NONE {
        THROW "invalid or expired virtual API key";
    };

    RETURN CREATE request_log CONTENT {
        request_id: $request_id,
        user: $key.user,
        virtual_api_key: $key.id,
        model_alias: $model_alias,
        provider: $provider,
        upstream_model: $upstream_model,
        status_code: $status_code,
        latency_ms: $latency_ms,
        stream: $stream,
        request_url: $request_url,
        error_message: $error_message,
        usage_input_tokens: $usage_input_tokens,
        usage_output_tokens: $usage_output_tokens
    };
} PERMISSIONS FULL;
