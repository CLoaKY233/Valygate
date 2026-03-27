# ValyMux

**Production-grade LLM Gateway, Proxy, and Observability Platform**

ValyMux is a single, self-hosted service that sits in front of your LLM providers. It proxies requests, routes by model name, enforces rate limits, manages prompts, and records every interaction for analysis — all in one Rust binary.

[![CI](https://github.com/CLoaKY233/Valymux/actions/workflows/ci.yml/badge.svg)](https://github.com/CLoaKY233/Valymux/actions/workflows/ci.yml)
[![Security Audit](https://github.com/CLoaKY233/Valymux/actions/workflows/audit.yml/badge.svg)](https://github.com/CLoaKY233/Valymux/actions/workflows/audit.yml)
[![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![CodeRabbit Pull Request Reviews](https://img.shields.io/coderabbit/prs/github/CLoaKY233/Valymux?utm_source=oss&utm_medium=github&utm_campaign=CLoaKY233%2FValymux&labelColor=171717&color=FF570A&link=https%3A%2F%2Fcoderabbit.ai&label=CodeRabbit+Reviews)](https://coderabbit.ai)

---

## Feature Status

| Feature | Status | Description |
|---------|--------|-------------|
| HTTP Proxy | Working | OpenAI-compatible `/v1/chat/completions` forwarded to the upstream provider |
| Streaming | Working | Server-sent events (SSE) streamed directly to the caller |
| Observability | Working | Per-request tracing, structured JSON logs, Prometheus-compatible metrics |
| Authentication | Working | Virtual API key generation and `RequireAuth` middleware |
| Model Routing | Planned | Route `gpt-4` to OpenAI, `claude-*` to Anthropic, etc. |
| Rate Limiting | Planned | Per-key and per-tenant request quotas |
| Prompt Management | Planned | Versioned prompts with A/B testing and deployment slots |

---

## Why ValyMux?

Most teams that consume multiple LLM providers end up writing the same plumbing repeatedly: key management, retry logic, cost tracking, logging. ValyMux packages all of that into a single OpenAI-compatible endpoint. Your applications point at ValyMux; ValyMux handles the rest.

- **Drop-in replacement.** Any OpenAI SDK works without modification — just change the base URL.
- **Zero-overhead proxy.** Written in Rust with Tokio; designed for low latency and high throughput.
- **Observability built in.** Every request is traced. Metrics are exposed in a format Prometheus can scrape.
- **Secrets handled safely.** Provider API keys are encrypted at rest with AES-256-GCM; the key derivation uses PBKDF2.

---

## Quick Start

### Prerequisites

- Rust 1.85 or later (`rustup update stable`)
- A running [SurrealDB](https://surrealdb.com/) instance or a Surreal Cloud endpoint

### Run Locally

```sh
git clone https://github.com/CLoaKY233/Valymux.git
cd Valymux

# Copy and edit the environment template
cp .env.example .env
$EDITOR .env

# Build and run
cargo run
```

The server starts on `http://0.0.0.0:3000` by default.

### Run with Docker

```sh
# Build the image
docker build -t valymux:latest .

# Run with environment variables
docker run --rm \
  --env-file .env \
  -p 3000:3000 \
  valymux:latest
```

### Run with Docker Compose

```yaml
# docker-compose.yml
services:
  valymux:
    build: .
    env_file: .env
    ports:
      - "3000:3000"
    restart: unless-stopped
```

```sh
docker compose up -d
```

---

## Configuration

All configuration is via environment variables. Copy `.env.example` to `.env` and adjust values.

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_HOST` | `0.0.0.0` | Interface to bind |
| `SERVER_PORT` | `3000` | Listening port |
| `LOG_FORMAT` | `compact` | `json` / `compact` / `pretty` |
| `RUST_LOG` | `valymux=info` | [EnvFilter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html) directive |
| `HTTP_TIMEOUT_SECS` | `300` | Outbound request timeout |
| `SURREAL_URL` | — | SurrealDB WebSocket endpoint |
| `SURREAL_NAMESPACE` | `main` | SurrealDB namespace |
| `SURREAL_DATABASE` | `main` | SurrealDB database |
| `SURREAL_USERNAME` | — | SurrealDB credentials |
| `SURREAL_PASSWORD` | — | SurrealDB credentials |
| `SURREAL_ENCRYPTION_KEY` | — | 32-byte hex key for secret encryption |

Generate a secure encryption key with:

```sh
openssl rand -hex 32
```

---

## API

ValyMux exposes an OpenAI-compatible API. Point any OpenAI SDK at `http://localhost:3000` and use a ValyMux virtual API key.

### Health Check

```
GET /
```

Returns `200 OK` when the server is running.

### Chat Completions

```
POST /v1/chat/completions
Authorization: Bearer <your-valymux-api-key>
Content-Type: application/json
```

The request and response schemas match the [OpenAI Chat Completions API](https://platform.openai.com/docs/api-reference/chat). Streaming (`"stream": true`) is supported.

---

## Project Structure

```
.
├── src/                        # Main binary crate (valymux)
│   ├── main.rs                 # Entry point, server start-up, graceful shutdown
│   ├── sys/                    # System-level concerns
│   │   ├── config.rs           # AppConfig loaded from environment via envy
│   │   ├── init.rs             # AppState construction, HTTP client, TCP listener
│   │   └── state.rs            # Arc<AppState> definition
│   └── rts/                    # Axum route handlers and middleware
│       ├── extractors.rs       # RequireAuth extractor
│       └── v1/                 # /v1/* handlers
├── crates/
│   ├── core/                   # Shared error types (AppError + IntoResponse)
│   ├── surrealdb/              # SurrealDB client, schema, models, crypto
│   └── telemetry/              # Tracing initialisation (JSON / pretty / compact)
└── docs/                       # Architecture and extended documentation
```

---

## Development

```sh
# Run tests
cargo test

# Lint (warnings treated as errors)
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt --all

# Check for known security advisories
cargo audit

# Check license and dependency policy
cargo deny check
```

---

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. For security issues, see [SECURITY.md](SECURITY.md).

---

## License

ValyMux is released under the [GNU Affero General Public License v3.0](LICENSE).

In short: you may use, modify, and distribute this software freely, but any modified version that you run as a network service must also be made available under the same license.
