# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## Overview

ValyMux is a production-grade, self-hosted LLM gateway written in Rust. It proxies requests to OpenAI, Anthropic, and Google Gemini with unified OpenAI-compatible endpoints, encrypted provider key management, virtual key scoping, and usage tracking.

---

## Project Structure & Module Organization

ValyMux is a Rust workspace with a binary crate (`src/`) and three library crates (`crates/`):

- **`src/`** — Main API binary (Axum HTTP server)
  - `main.rs` — Entry point, server startup, graceful shutdown
  - `sys/` — System concerns: config loading, app state initialization, shared dependencies
  - `rts/` — Routes and HTTP handlers: auth, `/v1` proxy, control plane endpoints
  - `svc/` — Service layer: provider adapters (OpenAI, Anthropic, Google), request translation
  - `lib.rs` — Exports shared modules for crate integration

- **`crates/core/`** — Shared application errors and response conversion used across workspace
  - Defines `AppError` enum and `IntoResponse` trait impl for Axum
  - Single source of truth for error shapes returned to clients

- **`crates/surrealdb/`** — Database access, schema, models, encryption, and persistence
  - `schema/` — SurrealQL files for tables, indices, seed data
  - `models/` — Rust types mirroring database records
  - `crypto.rs` — AES-256-GCM encryption for provider keys
  - `client.rs` — SurrealDB connection pool and query helpers

- **`crates/telemetry/`** — Tracing and logging initialization
  - Supports `json`, `compact`, and `pretty` output formats
  - Integrated with `src/sys/` for startup

---

## Key Request Flows

### Proxy Request (Chat Completions)

```text
POST /v1/chat/completions (Bearer token)
  → RequireAuth extractor validates virtual key
  → handler looks up provider credentials
  → src/svc/proxy/provider.rs translates OpenAI request → provider format
  → reqwest makes HTTP call to provider (OpenAI/Anthropic/Google)
  → provider response parsed & translated back to OpenAI format
  → response streamed (SSE) or returned as JSON
  → request_log record created in SurrealDB
```

### Virtual Key Scoping

Virtual keys are independent of real provider credentials. A virtual key:
- Is tied to a **user account** (derived from JWT)
- Can be restricted to specific **models**
- Maps to underlying **provider credentials** (encrypted in SurrealDB)
- Can be **revoked instantly** without affecting other keys

Implementation:
- Virtual keys are UUIDs stored in SurrealDB with scoping rules
- Provider credentials are encrypted with `SURREAL_ENCRYPTION_KEY` before storage
- Handler extracts credential ID from virtual key, decrypts provider secret at request time

---

## Navigation Guide — Where to Find Things

**Startup & Configuration:**
- `src/main.rs` — Server entry point, middleware stack, listener binding
- `src/sys/config.rs` — AppConfig struct, environment variable loading (envy)
- `src/sys/init.rs` — AppState construction, HTTP client setup, SurrealDB pool init
- `src/sys/state.rs` — `Arc<AppState>` definition and shared components

**HTTP Routing & Handlers:**
- `src/rts/mod.rs` — Router setup, middleware composition
- `src/rts/extractors.rs` — `RequireAuth` extractor for JWT validation
- `src/rts/v1/` — Chat completions endpoint, provider routing, usage tracking

**Provider Translation:**
- `src/svc/proxy/mod.rs` — `ProviderAdapter` trait and provider dispatch
- `src/svc/proxy/openai.rs` — OpenAI request/response translation
- `src/svc/proxy/anthropic.rs` — Anthropic request/response translation (supports structured content)
- `src/svc/proxy/google.rs` — Google Gemini request/response translation

**Database & Persistence:**
- `crates/surrealdb/src/schema/` — SurrealQL files (001_init.surql, 002_seed_models.surql)
- `crates/surrealdb/src/models/` — Rust types for users, credentials, virtual_keys, models, request_logs
- `crates/surrealdb/src/crypto.rs` — AES-256-GCM encryption/decryption for secrets

**Documentation (Source of Truth):**
- `documents/01_MVP_DEFINITION.md` — MVP scope, user flow, feature set
- `documents/04_ROADMAP_30_DAYS.md` — 30-day build roadmap with daily targets
- `documents/ARCHITECTURE.md` — System overview, design decisions, deployment patterns
- `documents/06_API_REFERENCE.md` — OpenAI-compatible endpoint specification
- `documents/09_SYSTEM_FLOW.md` — Detailed request lifecycle and error handling
- `documents/10_RUST_ARCHITECTURE.md` — Crate organization, trait hierarchies, async patterns
- `documents/03_MODEL_CATALOG.md` — Complete list of 21 launch models with exact specs
- `documents/ICP-Pain-Points-Market-Gap-USP-Moat-Pricing-and-Validation-Plan.md` — Product/market positioning

---

## Build, Test, and Development Commands

**Running the server:**
```bash
cargo run                    # Start API on 0.0.0.0:3000 (see .env.example)
cargo run --release         # Release-optimized binary
```

**Testing:**
```bash
cargo test                   # Run all tests (unit + integration)
cargo test --lib            # Unit tests only
cargo test --test '*'       # Integration tests only
cargo test test_name        # Run a single test by name
```

**Code quality:**
```bash
cargo clippy --all-targets --all-features -- -D warnings  # Lint check (required before PR)
cargo fmt --all             # Format all code (required before PR)
cargo fmt --all -- --check  # Check formatting without modifying
```

**Dependency & security audits:**
```bash
cargo audit                  # Check for known vulnerabilities
cargo deny check             # Check license policy and dependency security
cargo update                 # Update dependencies (monthly recommended)
```

**Release build:**
```bash
cargo build --release        # Production binary (target/release/valymux)
```

---

## Setup and Configuration

**Local Development Setup:**
1. Install Rust 1.85+ (stable): `rustup update stable`
2. Clone repo and navigate to project root
3. Copy `.env.example` to `.env` and edit:
   ```bash
   cp .env.example .env
   $EDITOR .env
   ```
4. Set `SURREAL_ENCRYPTION_KEY`:
   ```bash
   openssl rand -hex 32  # Generate 32-byte hex key
   ```
5. Start SurrealDB (local or cloud) and update `SURREAL_URL`, `SURREAL_NAMESPACE`, `SURREAL_DATABASE`
6. Run: `cargo run`

**Environment Variables:**
- `SERVER_HOST`, `SERVER_PORT` — Bind address (default `0.0.0.0:3000`)
- `LOG_FORMAT` — Output format: `json` / `compact` / `pretty` (default `compact`)
- `RUST_LOG` — Filter directive (default `valymux=info`). Examples: `valymux=debug`, `valymux=trace,tower_http=debug`
- `HTTP_TIMEOUT_SECS` — Upstream request timeout (default `300`)
- `SURREAL_URL`, `SURREAL_NAMESPACE`, `SURREAL_DATABASE` — SurrealDB connection (required)
- `SURREAL_USERNAME`, `SURREAL_PASSWORD` — SurrealDB credentials (optional; migration/bootstrap only)
- `VALYMUX_DB_SERVICE_KEY` — Bearer key for backend_service DB access used by the proxy (required for proxy operations)
- `SURREAL_ENCRYPTION_KEY` — 32-byte hex key for AES-256-GCM encryption of provider secrets (required, no default)

---

## Coding Style

**Naming Conventions:**
- Modules and files: `snake_case` (e.g., `provider_adapter.rs`, `request_handler.rs`)
- Types (structs, enums, traits): `PascalCase` (e.g., `AppState`, `ProviderAdapter`, `ProviderKind`)
- Constants: `UPPER_SNAKE_CASE` (e.g., `DEFAULT_TIMEOUT_SECS`)
- Variables & functions: `snake_case` (e.g., `extract_token()`, `api_key`)

**Code Organization:**
- Keep HTTP handlers thin — delegate business logic to service layer (`src/svc/`)
- Put provider-specific translation in `src/svc/proxy/` (separate file per provider)
- Database queries belong in `crates/surrealdb/` or service layer, not handlers
- Reusable error types in `crates/core/` — don't define per-module

**Formatting & Linting:**
- Follow `rustfmt.toml`: max_width = 100, Unix newlines, field-init shorthand, try shorthand
- All code must pass `cargo clippy --all-targets --all-features -- -D warnings` (no warnings)
- Format with `cargo fmt --all` before committing
- Target stable Rust 1.85 (avoid nightly-only features unless toolchain version is updated)

---

## Testing Guidelines

**Approach:**
- Place **unit tests** next to the code they exercise using `#[cfg(test)]` modules
- Add **integration tests** only for cross-module behavior (e.g., full request flow)
- Name tests after the **behavior** they verify, not implementation details
  - ✅ `test_returns_error_for_invalid_api_key()`
  - ❌ `test_jwt_parse()`

**Test Patterns:**
- Auth tests: Verify JWT validation, scoping, revocation
- Provider tests: Verify request translation (OpenAI → provider format → back)
- Database tests: Use real SurrealDB instance (integration tests)
- Error tests: Verify error messages are clear, don't leak secrets

**Coverage Focus:**
- Core auth flows (signup, signin, JWT validation)
- Provider adapter translation (edge cases: missing fields, nested arrays, streaming)
- Virtual key scoping (model restrictions, credential lookup)
- Proxy flow (request → response round-trip for each provider)

---

## MVP Scope & Roadmap

**MVP Vision (Target: April 27, 2026):**

ValyMux MVP is a self-hosted LLM gateway where a developer can:
1. Sign up and manage a personal account
2. Add provider API keys (OpenAI, Anthropic, Google Gemini) — encrypted at rest
3. Create virtual keys with model scoping
4. Proxy requests through a unified OpenAI-compatible endpoint (`/v1/chat/completions`)
5. View usage (tokens per model, per virtual key)

**MVP Includes:**
- Encrypted provider key vault (AES-256-GCM)
- Virtual keys with model-level scoping
- OpenAI-compatible proxy (streaming + non-streaming)
- Capability-aware model catalog (21 models × 3 providers)
- Basic usage tracking and request logs

**MVP Excludes:**
- Team/org management, rate limiting, load balancing
- Semantic caching, retry/fallback logic
- Custom model aliases, cost budgets, guardrails/PII masking
- MCP gateway, prompt management, SSO/SAML
- Embeddings, image generation, realtime/websocket APIs

**Build Phases** (from `documents/02_BUILD_PARTS.md`):
- **Part 1** — Backend fixes + Google provider (4 days)
- **Part 2** — Model catalog seed + usage endpoints (4 days)
- **Part 3** — Auth UI + key management UI (4 days)
- **Part 4** — Model catalog UI + playground (4 days)
- **Part 5** — Usage dashboard + polish (4 days)

See `documents/01_MVP_DEFINITION.md` and `documents/04_ROADMAP_30_DAYS.md` for detailed specs and daily targets.

---

## Commit & Pull Request Guidelines

**Commit Style:**
- Use short, imperative subject lines (≤50 chars)
- Prefix with scope when helpful: `Chore:`, `Feat:`, `Fix:`, `Docs:`
- Keep commits focused and reviewable (one logical change per commit)
- Examples:
  - `Fix: Handle Anthropic streaming edge case`
  - `Feat: Add Google Gemini provider adapter`
  - `Chore: Update Rust toolchain to 1.94`

**Pull Requests:**
- Follow `.github/PULL_REQUEST_TEMPLATE.md` — summary, type, change list, testing notes
- Link related issue if one exists
- Mention any API or schema changes
- Ensure CI passes (`cargo test`, `cargo clippy`, `cargo fmt`)
- Request review from project maintainers

---

## Security Practices

**Secret Management:**
- Provider API keys stored **encrypted** in SurrealDB (AES-256-GCM with `SURREAL_ENCRYPTION_KEY`)
- Virtual keys are **independent** of real credentials — they map to encrypted secrets by ID
- **Never** log secrets, API keys, or encrypted values to stdout/logs
- **Never** commit `.env` files or local keys to git (use `.env.example`)
- **Always** use `SURREAL_ENCRYPTION_KEY` — it's mandatory, no defaults

**Data Handling:**
- Request bodies can contain user messages but are only logged for debugging (redact in production)
- Response bodies should not contain secrets (provider errors sometimes leak)
- Usage logs track tokens and model names, not message content

**Dependency Security:**
- Run `cargo audit` and `cargo deny check` before releasing
- Keep dependencies up-to-date (`cargo update` monthly)
- Use workspace dependencies (defined in root `Cargo.toml`) for consistency
