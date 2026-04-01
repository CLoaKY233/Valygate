-- Virtual API Key Functions
-- PERMISSIONS WHERE $auth != NONE: $auth.id IS available inside the function body

DEFINE FUNCTION OVERWRITE fn::list_virtual_api_keys() {
    RETURN SELECT * FROM virtual_api_key
           WHERE user = $auth.id
           ORDER BY created_at ASC;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::get_virtual_api_key($id: record<virtual_api_key>) {
    RETURN (SELECT * FROM $id WHERE user = $auth.id)[0];
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::validate_model_aliases($aliases: array) {
    IF array::len($aliases) = 0 {
        RETURN true;
    };

    LET $existing = (
        SELECT VALUE alias FROM model_catalog
        WHERE user = $auth.id
          AND alias INSIDE $aliases
          AND enabled = true
          AND provider_credential.enabled = true
    );

    LET $missing = array::complement($aliases, $existing);

    IF array::len($missing) > 0 {
        THROW string::concat(
            "model alias(es) not found in your catalog: ",
            string::join(", ", $missing)
        );
    };

    RETURN true;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::create_virtual_api_key(
    $name: string,
    $key_prefix: string,
    $key_hash: string,
    $allowed_models: array,
    $tags: array,
    $expires_at: option<datetime>
) {
    IF array::len($allowed_models) > 0 {
        LET $existing_aliases = (
            SELECT VALUE alias FROM model_catalog
            WHERE user = $auth.id
              AND alias INSIDE $allowed_models
              AND enabled = true
              AND provider_credential.enabled = true
        );
        LET $missing = array::complement($allowed_models, $existing_aliases);
        IF array::len($missing) > 0 {
            THROW string::concat(
                "model alias(es) not found in your catalog: ",
                string::join(", ", $missing)
            );
        };
    };

    RETURN CREATE virtual_api_key CONTENT {
        user: $auth.id,
        name: $name,
        key_prefix: $key_prefix,
        key_hash: $key_hash,
        allowed_models: $allowed_models,
        tags: $tags,
        enabled: true,
        expires_at: $expires_at
    };
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::update_virtual_api_key(
    $id: record<virtual_api_key>,
    $name: string,
    $allowed_models: array,
    $tags: array,
    $enabled: bool,
    $expires_at: option<datetime>
) {
    LET $existing = (SELECT * FROM $id WHERE user = $auth.id)[0];
    IF $existing = NONE {
        THROW "virtual API key not found or access denied";
    };

    IF array::len($allowed_models) > 0 {
        LET $existing_aliases = (
            SELECT VALUE alias FROM model_catalog
            WHERE user = $auth.id
              AND alias INSIDE $allowed_models
              AND enabled = true
              AND provider_credential.enabled = true
        );
        LET $missing = array::complement($allowed_models, $existing_aliases);
        IF array::len($missing) > 0 {
            THROW string::concat(
                "model alias(es) not found in your catalog: ",
                string::join(", ", $missing)
            );
        };
    };

    RETURN (UPDATE $id MERGE {
        name: $name,
        allowed_models: $allowed_models,
        tags: $tags,
        enabled: $enabled,
        expires_at: $expires_at
    } WHERE user = $auth.id RETURN AFTER)[0];
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::delete_virtual_api_key($id: record<virtual_api_key>) {
    LET $before = (SELECT * FROM $id WHERE user = $auth.id)[0];
    IF $before = NONE {
        RETURN NONE;
    };

    DELETE $id WHERE user = $auth.id;

    RETURN $before;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";
