# ValyMux Setup Scripts

One-time setup tools for configuring a fresh ValyMux database. These scripts use the
[`surreal` CLI](https://surrealdb.com/install) and are designed to be idempotent, composable,
and usable in CI/CD pipelines.

---

## Prerequisites

| Requirement | Notes |
|-------------|-------|
| [Rust 1.85+](https://rustup.rs) | `rustup update stable` |
| [SurrealDB](https://surrealdb.com/install) | Local, Docker, or SurrealDB Cloud |
| `surreal` CLI | Installed with SurrealDB |

### Install the surreal CLI

```bash
# Linux / macOS
curl -sSf https://install.surrealdb.com | sh

# macOS via Homebrew
brew install surrealdb/tap/surreal

# Verify
surreal version
```

---

## Quick Start

```bash
# 1. Copy environment template
cp .env.example .env
$EDITOR .env   # fill in SURREAL_URL, SURREAL_NAMESPACE, SURREAL_DATABASE, credentials

# 2. Generate the encryption key (paste into .env as SURREAL_ENCRYPTION_KEY)
openssl rand -hex 32

# 3. Apply the database schema
./scripts/apply_schema.sh

# 4. Create the backend service grant (paste output into .env as VALYMUX_DB_SERVICE_KEY)
./scripts/generate_backend_grant.sh

# 5. Start the server
cargo run
```

---

## Scripts Reference

### `apply_schema.sh`

Applies all SurrealQL schema files to the database in the correct dependency order.

```bash
./scripts/apply_schema.sh
```

**Safe to re-run** — all statements use `DEFINE ... OVERWRITE` or `IF NOT EXISTS`.
Run this again whenever schema files are updated (e.g., after pulling new changes).

**Required environment variables:**

| Variable | Example | Description |
|----------|---------|-------------|
| `SURREAL_URL` | `ws://localhost:8000` | SurrealDB WebSocket endpoint |
| `SURREAL_NAMESPACE` | `valymux` | Namespace |
| `SURREAL_DATABASE` | `valymux` | Database |
| `SURREAL_USERNAME` | `root` | Admin username (or use `SURREAL_TOKEN`) |
| `SURREAL_PASSWORD` | `root` | Admin password |

---

### `generate_backend_grant.sh`

Creates the `backend_service` BEARER grant used by the ValyMux server to fetch encrypted
provider credentials from SurrealDB. Requires root/admin credentials.

```bash
./scripts/generate_backend_grant.sh
```

The script prints a single line to **stdout**:

```
VALYMUX_DB_SERVICE_KEY=surreal-access-...
```

Copy this into your `.env` file. **The key is printed once and cannot be retrieved again.**

**Grant expiry:** 30 days. Set a calendar reminder and rotate before expiry (see below).

---

## Grant Rotation

The `VALYMUX_DB_SERVICE_KEY` expires every 30 days. To rotate:

1. Generate a new grant:
   ```bash
   ./scripts/generate_backend_grant.sh
   ```

2. Update `VALYMUX_DB_SERVICE_KEY` in `.env` with the new key.

3. Restart the server:
   ```bash
   cargo run   # or restart your systemd/Docker service
   ```

4. Revoke the old grant (shown at the end of `generate_backend_grant.sh`):
   ```sql
   -- Run in Surrealist or via surreal sql
   ACCESS backend_service ON DATABASE REVOKE GRANT <old-grant-id>;
   ```

If a grant is compromised, immediately revoke it and generate a replacement.

---

## Environment Variables Reference

All variables are read from `.env` (if present) or the current environment.

| Variable | Required | Description |
|----------|----------|-------------|
| `SURREAL_URL` | Yes | SurrealDB endpoint (e.g. `ws://localhost:8000`) |
| `SURREAL_NAMESPACE` | Yes | Namespace (e.g. `valymux`) |
| `SURREAL_DATABASE` | Yes | Database (e.g. `valymux`) |
| `SURREAL_USERNAME` | If no token | Admin username |
| `SURREAL_PASSWORD` | If no token | Admin password |
| `SURREAL_TOKEN` | Alternative | Bearer token (overrides username/password) |
| `SURREAL_ENCRYPTION_KEY` | Yes (server) | 32-byte hex key for AES-256-GCM provider key encryption |
| `VALYMUX_DB_SERVICE_KEY` | Yes (server) | Backend service BEARER grant (from `generate_backend_grant.sh`) |

---

## Troubleshooting

**`surreal: command not found`**
Install the SurrealDB CLI: https://surrealdb.com/install

**`Failed to create backend_service grant` / schema errors**
Run `apply_schema.sh` before `generate_backend_grant.sh`. The `backend_service` ACCESS must
be defined before a grant can be created.

**Server fails to start with `VALYMUX_DB_SERVICE_KEY` error**
Run `./scripts/generate_backend_grant.sh` and set the output key in `.env`.

**Grant expired (server auth fails after 30 days)**
Follow the [Grant Rotation](#grant-rotation) procedure above.
