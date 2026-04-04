#!/usr/bin/env bash
# generate_backend_grant.sh — Create the backend_service BEARER grant for ValyMux.
#
# The backend_service grant allows the ValyMux server to fetch encrypted provider
# credentials from SurrealDB. The raw grant key is stored as VALYMUX_DB_SERVICE_KEY
# in your .env file and is the only way the server can decrypt provider API keys.
#
# Usage:
#   ./scripts/generate_backend_grant.sh
#
# Environment variables (set in .env or export manually):
#   SURREAL_URL        — SurrealDB endpoint  (e.g. ws://localhost:8000)
#   SURREAL_NAMESPACE  — Namespace           (e.g. valymux)
#   SURREAL_DATABASE   — Database            (e.g. valymux)
#   SURREAL_USERNAME   — Admin username      (default: root)
#   SURREAL_PASSWORD   — Admin password      (default: root)
#
# Security notes:
#   - This script requires root/admin SurrealDB credentials.
#   - The generated key grants access to read encrypted provider credentials.
#     It does NOT grant access to decrypt them (that requires SURREAL_ENCRYPTION_KEY).
#   - The key is printed ONCE. Store it immediately in your .env file.
#   - Grants expire in 30 days. Run this script again to rotate.
#   - To revoke a compromised key: run the REVOKE command shown at the end.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=scripts/common.sh
. "$SCRIPT_DIR/common.sh"

# ─────────────────────────────────────────────────────────
# Preflight
# ─────────────────────────────────────────────────────────
load_env_file
check_surreal_cli
require_env SURREAL_URL SURREAL_NAMESPACE SURREAL_DATABASE

log_info "Generating backend_service BEARER grant"
log_info "Database: ${SURREAL_URL} / ${SURREAL_NAMESPACE} / ${SURREAL_DATABASE}"
echo "" >&2

# ─────────────────────────────────────────────────────────
# Create a temporary SurrealQL file for the GRANT statement.
# Using a temp file avoids the key appearing in process list (ps aux).
# ─────────────────────────────────────────────────────────
TMPFILE="$(mktemp /tmp/valymux_grant_XXXXXX.surql)"
trap 'rm -f "$TMPFILE"' EXIT

cat > "$TMPFILE" <<'EOF'
ACCESS backend_service ON DATABASE GRANT FOR USER root DURATION FOR GRANT 30d;
EOF

# ─────────────────────────────────────────────────────────
# Execute and capture output
# ─────────────────────────────────────────────────────────
RAW_OUTPUT=""
if ! RAW_OUTPUT="$(surreal_sql "$TMPFILE" 2>&1)"; then
    log_error "Failed to create backend_service grant."
    log_error ""
    log_error "Common causes:"
    log_error "  1. Schema not yet applied — run ./scripts/apply_schema.sh first."
    log_error "  2. Wrong SURREAL_USERNAME / SURREAL_PASSWORD (root credentials required)."
    log_error "  3. SurrealDB is not reachable at ${SURREAL_URL}."
    log_error ""
    log_error "Raw error output:"
    printf '%s\n' "$RAW_OUTPUT" >&2
    exit 1
fi

# ─────────────────────────────────────────────────────────
# Parse the bearer key from the response.
# SurrealDB returns a JSON object containing the key field.
# Example: [{"ac":"backend_service","id":"...","key":"surreal-access-...","..."}]
# ─────────────────────────────────────────────────────────
GRANT_KEY=""

# Try jq first (clean parse)
if command -v jq &>/dev/null; then
    GRANT_KEY="$(printf '%s' "$RAW_OUTPUT" | jq -r '.[0].result[0].key // empty' 2>/dev/null || true)"
fi

# Fallback: grep for the key field pattern
if [ -z "$GRANT_KEY" ]; then
    GRANT_KEY="$(printf '%s' "$RAW_OUTPUT" | grep -oP '"key"\s*:\s*"\K[^"]+' | head -1 || true)"
fi

if [ -z "$GRANT_KEY" ]; then
    log_error "Grant was created but the key could not be parsed from the response."
    log_error "Raw SurrealDB output:"
    printf '%s\n' "$RAW_OUTPUT" >&2
    log_error ""
    log_error "Look for the 'key' field in the output above and set it manually:"
    log_error "  VALYMUX_DB_SERVICE_KEY=<key>"
    exit 1
fi

# ─────────────────────────────────────────────────────────
# Extract the grant ID for the revocation instructions
# ─────────────────────────────────────────────────────────
GRANT_ID=""
if command -v jq &>/dev/null; then
    GRANT_ID="$(printf '%s' "$RAW_OUTPUT" | jq -r '.[0].result[0].id // empty' 2>/dev/null || true)"
fi
if [ -z "$GRANT_ID" ]; then
    GRANT_ID="$(printf '%s' "$RAW_OUTPUT" | grep -oP '"id"\s*:\s*"\K[^"]+' | head -1 || true)"
fi

# ─────────────────────────────────────────────────────────
# Output
# ─────────────────────────────────────────────────────────
echo "" >&2
log_info "Backend service grant created successfully."
echo "" >&2

printf "${_BOLD}${_GREEN}┌─────────────────────────────────────────────────────────────┐${_RESET}\n" >&2
printf "${_BOLD}${_GREEN}│  Copy the following line into your .env file:               │${_RESET}\n" >&2
printf "${_BOLD}${_GREEN}└─────────────────────────────────────────────────────────────┘${_RESET}\n" >&2
echo ""
printf "VALYMUX_DB_SERVICE_KEY=%s\n" "$GRANT_KEY"
echo ""

log_warn "This key will NOT be shown again. Store it immediately."
log_warn "The grant expires in 30 days. Re-run this script to rotate."
echo "" >&2

if [ -n "$GRANT_ID" ]; then
    log_info "To revoke this grant if compromised, run in Surrealist/CLI:"
    printf "  ACCESS backend_service ON DATABASE REVOKE GRANT %s;\n" "$GRANT_ID" >&2
    echo "" >&2
fi

log_info "Once .env is updated, restart the server: cargo run"
