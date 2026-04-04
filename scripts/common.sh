#!/usr/bin/env bash
# common.sh — Shared helpers for ValyMux setup scripts.
# Source this file at the top of each script: . "$(dirname "$0")/common.sh"

# ─────────────────────────────────────────────────────────
# Terminal colours (disabled when not a tty)
# ─────────────────────────────────────────────────────────
if [ -t 2 ]; then
    _RESET='\033[0m'
    _BOLD='\033[1m'
    _GREEN='\033[0;32m'
    _YELLOW='\033[0;33m'
    _RED='\033[0;31m'
    _CYAN='\033[0;36m'
else
    _RESET='' _BOLD='' _GREEN='' _YELLOW='' _RED='' _CYAN=''
fi

log_info()  { printf "${_GREEN}[INFO]${_RESET}  %s\n" "$*" >&2; }
log_warn()  { printf "${_YELLOW}[WARN]${_RESET}  %s\n" "$*" >&2; }
log_error() { printf "${_RED}[ERROR]${_RESET} %s\n" "$*" >&2; }
log_step()  { printf "${_CYAN}${_BOLD}  →${_RESET} %s\n" "$*" >&2; }

# ─────────────────────────────────────────────────────────
# load_env_file — Source .env from project root if present.
# Existing environment variables are NOT overwritten.
# ─────────────────────────────────────────────────────────
load_env_file() {
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local project_root
    project_root="$(dirname "$script_dir")"
    local env_file="$project_root/.env"

    if [ -f "$env_file" ]; then
        log_step "Loading environment from $env_file"
        # Export only variables that are not already set
        while IFS='=' read -r key value; do
            # Skip comments and blank lines
            [[ "$key" =~ ^[[:space:]]*# ]] && continue
            [[ -z "$key" ]] && continue
            key="${key// /}"        # strip spaces from key
            value="${value%%#*}"    # strip inline comments
            value="${value%"${value##*[![:space:]]}"}"  # rtrim
            [ -z "${!key+x}" ] && export "$key=$value"
        done < <(grep -v '^[[:space:]]*#' "$env_file" | grep '=')
    fi
}

# ─────────────────────────────────────────────────────────
# require_env VAR [VAR...] — Abort if any variable is unset or empty.
# ─────────────────────────────────────────────────────────
require_env() {
    local missing=()
    for var in "$@"; do
        if [ -z "${!var:-}" ]; then
            missing+=("$var")
        fi
    done
    if [ ${#missing[@]} -gt 0 ]; then
        log_error "The following required environment variables are not set:"
        for var in "${missing[@]}"; do
            log_error "  $var"
        done
        log_error "Copy .env.example to .env and fill in the values, or export them manually."
        exit 1
    fi
}

# ─────────────────────────────────────────────────────────
# check_surreal_cli — Verify the surreal binary is available.
# ─────────────────────────────────────────────────────────
check_surreal_cli() {
    if ! command -v surreal &>/dev/null; then
        log_error "'surreal' CLI not found in PATH."
        log_error "Install SurrealDB: https://surrealdb.com/install"
        exit 1
    fi
    local version
    version="$(surreal version 2>&1 | head -1 || true)"
    log_step "surreal CLI: $version"
}

# ─────────────────────────────────────────────────────────
# surreal_sql FILE — Execute a SurrealQL file against the configured database.
# Reads: SURREAL_URL, SURREAL_NAMESPACE, SURREAL_DATABASE,
#        SURREAL_USERNAME, SURREAL_PASSWORD (or SURREAL_TOKEN for token auth).
# ─────────────────────────────────────────────────────────
surreal_sql() {
    local file="$1"
    local auth_args=()

    if [ -n "${SURREAL_TOKEN:-}" ]; then
        auth_args=(--token "$SURREAL_TOKEN")
    else
        auth_args=(--user "${SURREAL_USERNAME:-root}" --pass "${SURREAL_PASSWORD:-root}")
    fi

    surreal sql \
        --endpoint "${SURREAL_URL}" \
        --namespace "${SURREAL_NAMESPACE}" \
        --database "${SURREAL_DATABASE}" \
        "${auth_args[@]}" \
        --hide-welcome \
        < "$file"
}
