-- Provider Credential Functions
-- PERMISSIONS WHERE $auth != NONE: $auth.id IS available inside the function body

DEFINE FUNCTION OVERWRITE fn::list_provider_credentials() {
    RETURN SELECT * FROM provider_credential
           WHERE user = $auth.id
           ORDER BY created_at ASC;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::get_provider_credential($id: record<provider_credential>) {
    RETURN (SELECT * FROM $id WHERE user = $auth.id)[0];
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

-- encrypted_api_key must be encrypted by Rust before calling this
DEFINE FUNCTION OVERWRITE fn::create_provider_credential(
    $provider: string,
    $label: string,
    $encrypted_api_key: string,
    $tags: array
) {
    RETURN CREATE provider_credential CONTENT {
        user: $auth.id,
        provider: $provider,
        label: $label,
        encrypted_api_key: $encrypted_api_key,
        tags: $tags,
        enabled: true,
        sync_status: "pending",
        model_count: 0
    };
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::update_provider_credential(
    $id: record<provider_credential>,
    $label: string,
    $tags: array,
    $enabled: bool,
    $encrypted_api_key: option<string>
) {
    LET $existing = (SELECT * FROM $id WHERE user = $auth.id)[0];
    IF $existing = NONE {
        THROW "provider credential not found or access denied";
    };

    IF $encrypted_api_key != NONE {
        LET $updated = (UPDATE $id MERGE {
            label: $label,
            tags: $tags,
            enabled: $enabled,
            encrypted_api_key: $encrypted_api_key,
            sync_status: "pending",
            sync_error: NONE
        } WHERE user = $auth.id RETURN AFTER)[0];
        RETURN $updated;
    };

    RETURN (UPDATE $id MERGE {
        label: $label,
        tags: $tags,
        enabled: $enabled
    } WHERE user = $auth.id RETURN AFTER)[0];
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::delete_provider_credential($id: record<provider_credential>) {
    LET $before = (SELECT * FROM $id WHERE user = $auth.id)[0];
    IF $before = NONE {
        RETURN NONE;
    };

    DELETE supports WHERE in = $id;
    DELETE virtual_key_route WHERE out = $id;
    DELETE $id WHERE user = $auth.id;

    RETURN $before;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";
