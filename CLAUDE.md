# ValyGate Project Context

ValyGate is an all-in-one LLM gateway, proxy, prompt manager, and observability platform. It is designed to centralize access to multiple LLM providers, manage prompts, and provide comprehensive tracking and analysis of LLM interactions.

## Technical Stack
- **Language:** Rust (Edition 2024)
- **Web Framework:** [Axum](https://github.com/tokio-rs/axum)
- **Async Runtime:** [Tokio](https://tokio.rs/)
- **Serialization:** [Serde](https://serde.rs/)
- **HTTP Client:** [Reqwest](https://github.com/seanmonstar/reqwest)
- **Logging/Tracing:** [Tracing](https://github.com/tokio-rs/tracing)
- **Error Handling:** `anyhow` and `thiserror`
- **Configuration:** `envy` (environment variables to typesafe structs)

## Project Structure
The project is organized as a Rust workspace:

- **`src/` (Main Crate):**
  - `main.rs`: Application entry point, server initialization, and graceful shutdown logic.
  - `lib.rs`: Module declarations (`sys`, `rts`).
  - `sys/`: System-level logic.
    - `config.rs`: `AppConfig` struct and environment variable mapping.
    - `init.rs`: Initialization logic for the `AppState`, HTTP client, and TCP listener.
    - `state.rs`: `AppState` shared across request handlers.
  - `rts/`: Runtime services and request handlers.
    - `root.rs`: Basic health-check/root handler.
- **`crates/core/`:**
  - `error.rs`: Centralized error handling using `AppError` and Axum's `IntoResponse`.
- **`crates/telemetry/`:**
  - `init.rs`: Tracing initialization with support for `Json`, `Pretty`, and `Compact` log formats.
  - `models.rs`: Configuration models for logging.

## Development Workflows

### Environment Configuration
ValyGate uses environment variables for configuration. Key variables include:
- `SERVER_HOST`: Host to bind the server to (default: `0.0.0.0`).
- `SERVER_PORT`: Port to listen on (default: `3000`).
- `LOG_FORMAT`: Format of logs (`pretty`, `json`, `compact`).
- `LOG_LEVEL`: Tracing filter level (e.g., `info`, `debug`).

### Building and Running
- **Build:** `cargo build`
- **Run:** `cargo run`
- **Test:** `cargo test`
- **Lint:** `cargo clippy`
- **Format:** `cargo fmt`

## Coding Conventions
- **Error Handling:** Use `AppError` from `crates/core` for user-facing errors. Use `anyhow` for internal/unexpected errors.
- **State Management:** Shared state is managed via `Arc<AppState>` and passed to Axum handlers using `.with_state()`.
- **Tracing:** Use the `tracing` macros (`info!`, `warn!`, `error!`, `debug!`) throughout the codebase for observability.
- **Graceful Shutdown:** The server is configured to handle `SIGINT` and `SIGTERM` for graceful termination.
