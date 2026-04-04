#!/usr/bin/env bash
# apply_schema.sh — Apply all ValyMux SurrealQL schema files in the correct dependency order.
#
# Usage:
#   ./scripts/apply_schema.sh
#
# Environment variables (set in .env or export manually):
#   SURREAL_URL        — SurrealDB endpoint  (e.g. ws://localhost:8000)
#   SURREAL_NAMESPACE  — Namespace           (e.g. valymux)
#   SURREAL_DATABASE   — Database            (e.g. valymux)
#   SURREAL_USERNAME   — Admin username      (default: root)
#   SURREAL_PASSWORD   — Admin password      (default: root)
#   SURREAL_TOKEN      — Bearer token        (alternative to username/password)
#
# This script is idempotent — all schema statements use DEFINE ... OVERWRITE or
# IF NOT EXISTS, so it is safe to re-run on an existing database.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
SCHEMA_DIR="$PROJECT_ROOT/crates/surrealdb/schema"

# shellcheck source=scripts/common.sh
. "$SCRIPT_DIR/common.sh"

# ─────────────────────────────────────────────────────────
# Preflight
# ─────────────────────────────────────────────────────────
load_env_file
check_surreal_cli
require_env SURREAL_URL SURREAL_NAMESPACE SURREAL_DATABASE

log_info "Applying ValyMux schema to ${SURREAL_URL} / ${SURREAL_NAMESPACE} / ${SURREAL_DATABASE}"

# ─────────────────────────────────────────────────────────
# Schema application order — dependencies must come first.
#
# Rule of thumb:
#   1. Access definitions before the tables they govern
#   2. Tables before the relations that reference them
#   3. Tables before the functions that operate on them
# ─────────────────────────────────────────────────────────
SCHEMA_FILES=(
    # Auth / access definitions
    "auth/001_account.surql"
    "auth/004_backend_service.surql"

    # Core tables (auth tables reference these)
    "user/001_user.surql"
    "auth/002_virtual_api_key.surql"

    # Auth that references the tables above
    "auth/003_virtual_key_access.surql"

    # Domain tables
    "provider/001_provider_credential.surql"
    "request/001_request_log.surql"
    "model/001_model_definition.surql"

    # Relation tables (reference domain tables)
    "model/002_supports.surql"
    "model/003_virtual_key_route.surql"

    # Functions (reference all tables above)
    "functions/001_user_functions.surql"
    "functions/002_provider_credential_functions.surql"
    "functions/003_virtual_api_key_functions.surql"
    "functions/004_model_catalog_functions.surql"
    "functions/005_proxy_functions.surql"
)

# ─────────────────────────────────────────────────────────
# Apply each file
# ─────────────────────────────────────────────────────────
APPLIED=0
FAILED=0

for relative_path in "${SCHEMA_FILES[@]}"; do
    full_path="$SCHEMA_DIR/$relative_path"

    if [ ! -f "$full_path" ]; then
        log_error "Schema file not found: $full_path"
        FAILED=$((FAILED + 1))
        continue
    fi

    log_step "Applying $relative_path"
    if surreal_sql "$full_path" > /dev/null 2>&1; then
        APPLIED=$((APPLIED + 1))
    else
        log_error "Failed to apply $relative_path"
        # Re-run with visible output for diagnostics
        surreal_sql "$full_path" || true
        FAILED=$((FAILED + 1))
    fi
done

# ─────────────────────────────────────────────────────────
# Summary
# ─────────────────────────────────────────────────────────
echo ""
if [ "$FAILED" -eq 0 ]; then
    log_info "Schema applied successfully ($APPLIED files)."
    log_info "Next step: run ./scripts/generate_backend_grant.sh to create the backend service key."
else
    log_error "Schema application completed with $FAILED failure(s) (applied $APPLIED files)."
    exit 1
fi
