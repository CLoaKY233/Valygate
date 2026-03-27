# Repository Guidelines

## Project Structure & Module Organization
ValyMux is a Rust workspace. The main binary lives in `src/`, with `src/main.rs` starting Axum and `src/lib.rs` exposing shared modules. Runtime code is split by concern: `src/sys/` for config, startup, and shared state; `src/rts/` for routes and extractors; and `src/svc/proxy/` for provider adapters. Workspace crates live under `crates/`: `crates/core/` for shared errors, `crates/surrealdb/` for schema/models/crypto, and `crates/telemetry/` for tracing setup. The `documents/` directory is the source of truth for architecture, API shapes, build parts, and test cases.

## Crate Responsibilities
- `crates/core/` - shared application errors and response conversion used across the workspace.
- `crates/surrealdb/` - database access, schema loading, model definitions, crypto helpers, and persistence logic.
- `crates/telemetry/` - tracing initialization and logging configuration for `json`, `compact`, and `pretty` output.
- `src/` - the main API binary, HTTP handlers, proxy adapters, and startup wiring that compose the crates above.

## Navigation Guide
- Start with `src/main.rs` for the server entry point and request middleware.
- Use `src/sys/` when changing config, state, or bootstrapping.
- Use `src/rts/` for request routing, auth extractors, and `/v1` handlers.
- Use `src/svc/proxy/` for provider translation and upstream request handling.
- Use `crates/surrealdb/schema/` when changing tables, seed data, or persistence rules.
- Use `documents/ARCHITECTURE.md`, `documents/09_SYSTEM_FLOW.md`, and `documents/06_API_REFERENCE.md` when you need the intended behavior before editing code.

## Build, Test, and Development Commands
- `cargo run` - start the API locally on `0.0.0.0:3000`.
- `cargo build --release` - build the production binary.
- `cargo test` - run the test suite.
- `cargo clippy --all-targets --all-features -- -D warnings` - enforce lint cleanliness.
- `cargo fmt --all` - format the workspace with the repo `rustfmt.toml`.
- `cargo audit` and `cargo deny check` - check dependency risk and license policy.

## Coding Style & Naming Conventions
Follow the repository `rustfmt.toml`: `max_width = 100`, Unix newlines, field-init shorthand, and try shorthand. Keep modules and files in `snake_case`, types in `PascalCase`, and constants in `UPPER_SNAKE_CASE`. Keep handlers thin and put provider-specific translation in `src/svc/proxy/`. The docs target stable Rust 1.85 compatibility, so avoid nightly-only features unless a change explicitly updates the toolchain plan.

## Testing Guidelines
Use `cargo test` for unit and integration tests. Place unit tests next to the code they exercise with `#[cfg(test)]`; add integration tests only when cross-module behavior matters. Name tests after the behavior they verify, not the implementation detail. For new work, map coverage to the docs in `documents/08_TEST_CASES.md`, especially auth, provider management, virtual keys, and proxy flows.

## MVP Direction
The near-term MVP, described in `documents/01_MVP_DEFINITION.md`, is a self-hosted LLM gateway where a developer can sign up, add BYOK provider credentials, create scoped virtual keys, configure models with capability-aware controls, proxy through `/v1/chat/completions`, and review basic usage. The launch providers are exactly three: OpenAI, Anthropic, and Google Gemini. The intended MVP flow is: self-host via `docker compose up`, sign up, add provider keys, see a catalog filtered to usable models, open a model-specific playground, create a virtual key, proxy traffic, and inspect usage logs.

Keep the launch scope tight. In the doc, the MVP includes encrypted provider keys, virtual-key scoping, an OpenAI-compatible proxy, a capability-aware model catalog, a visual config playground, and usage tracking. It excludes team/org management, semantic caching, retry/fallback logic, rate limiting, load balancing, guardrails/PII masking, MCP gateway work, prompt management, SSO/SAML, embeddings, image generation, realtime/websocket APIs, cost budgets, and custom model aliases.

## Commit & Pull Request Guidelines
Recent history uses short imperative subjects, often with a scope or prefix like `Chore:`. Keep commits focused and easy to review. PRs should follow `.github/PULL_REQUEST_TEMPLATE.md`: include a summary, type of change, change list, testing notes, and checklist completion. Link the related issue when possible, mention any API or schema changes, and add screenshots for UI work.

## Security & Configuration Tips
Configuration comes from environment variables. Copy `.env.example` to `.env`, set SurrealDB credentials, and generate `SURREAL_ENCRYPTION_KEY` with `openssl rand -hex 32`. The docs call for encrypted provider keys, virtual-key scoping, and no secret leakage in logs or responses. Do not commit keys, tokens, or local database endpoints.
