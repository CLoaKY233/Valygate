-- Virtual API Key Functions
-- Virtual keys own model scope, and may also carry explicit alias-to-credential routes.

DEFINE FUNCTION OVERWRITE fn::get_virtual_api_key($id: record<virtual_api_key>) {
    RETURN (
        SELECT
            *,
            (
                SELECT
                    model_alias,
                    out AS provider_credential_id,
                    out.provider AS provider,
                    out.label AS provider_label
                FROM virtual_key_route
                WHERE in = id
                ORDER BY model_alias ASC
            ) AS routes
        FROM $id
        WHERE user = $auth.id
    )[0];
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::get_virtual_api_key($id: record<virtual_api_key>) {
    RETURN (
        SELECT
            *,
            (
                SELECT VALUE {
                    model_alias: model_alias,
                    provider_credential_id: out,
                    provider: out.provider,
                    provider_label: out.label
                }
                FROM virtual_key_route
                WHERE in = id
                ORDER BY model_alias ASC
            ) AS routes
        FROM $id
        WHERE user = $auth.id
    )[0];
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::validate_model_aliases($aliases: array) {
    IF array::len($aliases) = 0 {
        RETURN true;
    };

    LET $existing = (
        SELECT VALUE alias FROM model_definition
        WHERE alias INSIDE $aliases
          AND id INSIDE (
              SELECT VALUE out FROM supports
              WHERE in INSIDE (
                  SELECT VALUE id FROM provider_credential
                  WHERE user = $auth.id
                    AND enabled = true
              )
          )
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

DEFINE FUNCTION OVERWRITE fn::validate_virtual_key_routes($routes: array, $allowed_models: array) {
    FOR $route IN $routes {
        LET $cred = (
            SELECT * FROM $route.provider_credential_id
            WHERE user = $auth.id
              AND enabled = true
        )[0];
        IF $cred = NONE {
            THROW "provider credential not found or access denied";
        };

        IF array::len($allowed_models) > 0 AND !($route.model_alias INSIDE $allowed_models) {
            THROW string::concat(
                "route alias is outside virtual key scope: ",
                $route.model_alias
            );
        };

        LET $model = (SELECT * FROM model_definition WHERE alias = $route.model_alias LIMIT 1)[0];
        IF $model = NONE {
            THROW string::concat("model alias not found: ", $route.model_alias);
        };

        LET $support = (
            SELECT * FROM supports
            WHERE in = $route.provider_credential_id
              AND out = $model.id
            LIMIT 1
        )[0];

        IF $support = NONE {
            THROW string::concat(
                "provider credential does not support model alias: ",
                $route.model_alias
            );
        };
    };

    RETURN true;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::replace_virtual_key_routes(
    $key_id: record<virtual_api_key>,
    $routes: array
) {
    DELETE virtual_key_route WHERE in = $key_id;

    FOR $route IN $routes {
        LET $provider_credential_id = $route.provider_credential_id;
        RELATE $key_id->virtual_key_route->$provider_credential_id
            SET model_alias = $route.model_alias;
    };

    RETURN (
        SELECT
            model_alias,
            out as provider_credential_id,
            out.provider as provider,
            out.label as provider_label
        FROM virtual_key_route
        WHERE in = $key_id
        ORDER BY model_alias ASC
    );
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::create_virtual_api_key(
    $name: string,
    $key_prefix: string,
    $key_hash: string,
    $allowed_models: array,
    $tags: array,
    $expires_at: option<datetime>,
    $routes: array
) {
    fn::validate_model_aliases($allowed_models);
    fn::validate_virtual_key_routes($routes, $allowed_models);

    LET $created = (
        CREATE virtual_api_key CONTENT {
            user: $auth.id,
            name: $name,
            key_prefix: $key_prefix,
            key_hash: $key_hash,
            allowed_models: $allowed_models,
            tags: $tags,
            enabled: true,
            expires_at: $expires_at
        }
    )[0];

    LET $created_routes = fn::replace_virtual_key_routes($created.id, $routes);

    RETURN $created + {
        routes: $created_routes
    };
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::update_virtual_api_key(
    $id: record<virtual_api_key>,
    $name: string,
    $allowed_models: array,
    $tags: array,
    $enabled: bool,
    $expires_at: option<datetime>,
    $routes: array
) {
    LET $existing = (SELECT * FROM $id WHERE user = $auth.id)[0];
    IF $existing = NONE {
        THROW "virtual API key not found or access denied";
    };

    fn::validate_model_aliases($allowed_models);
    fn::validate_virtual_key_routes($routes, $allowed_models);

    LET $updated = (
        UPDATE $id MERGE {
            name: $name,
            allowed_models: $allowed_models,
            tags: $tags,
            enabled: $enabled,
            expires_at: $expires_at
        }
        WHERE user = $auth.id
        RETURN AFTER
    )[0];

    LET $updated_routes = fn::replace_virtual_key_routes($id, $routes);

    RETURN $updated + {
        routes: $updated_routes
    };
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::delete_virtual_api_key($id: record<virtual_api_key>) {
    LET $before = (SELECT * FROM $id WHERE user = $auth.id)[0];
    IF $before = NONE {
        RETURN NONE;
    };

    DELETE virtual_key_route WHERE in = $id;
    DELETE $id WHERE user = $auth.id;

    RETURN $before;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";
