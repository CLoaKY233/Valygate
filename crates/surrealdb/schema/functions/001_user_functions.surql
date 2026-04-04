-- User Functions
-- PERMISSIONS WHERE $auth != NONE: $auth.id IS available inside the function body

DEFINE FUNCTION OVERWRITE fn::get_current_user() {
    -- Explicit column list intentionally excludes the password hash.
    LET $user = (
        SELECT id, name, email, enabled, created_at, updated_at
        FROM $auth.id
    )[0];
    IF $user = NONE {
        THROW "authenticated user not found";
    };
    IF $user.enabled = false {
        THROW "account is disabled";
    };
    RETURN $user;
} PERMISSIONS WHERE $auth != NONE AND $access = "account";

DEFINE FUNCTION OVERWRITE fn::update_profile($changed_name: string) {
    RETURN (UPDATE user SET name = $changed_name WHERE id = $auth.id RETURN AFTER)[0];
} PERMISSIONS WHERE $auth != NONE AND $access = "account";
