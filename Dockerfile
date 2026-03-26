# =============================================================================
# Stage 1 — Dependency cache
# Pre-fetch and compile dependencies separately so that source-only changes
# do not invalidate the dependency layer.
# =============================================================================
FROM rust:1.85-slim-bookworm AS deps

WORKDIR /build

# Install build essentials needed by some crates (e.g. ring, surrealdb).
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests and lockfile only — source is excluded on purpose.
COPY Cargo.toml Cargo.lock ./
COPY crates/core/Cargo.toml      crates/core/Cargo.toml
COPY crates/surrealdb/Cargo.toml crates/surrealdb/Cargo.toml
COPY crates/telemetry/Cargo.toml crates/telemetry/Cargo.toml

# Create stub lib/main files so `cargo build` can resolve the dependency graph
# without any real source code.
RUN mkdir -p src \
    crates/core/src \
    crates/surrealdb/src \
    crates/telemetry/src \
 && echo 'fn main() {}' > src/main.rs \
 && touch src/lib.rs \
 && touch crates/core/src/lib.rs \
 && touch crates/surrealdb/src/lib.rs \
 && touch crates/telemetry/src/lib.rs

RUN cargo build --release --locked \
 && rm -rf src crates/*/src

# =============================================================================
# Stage 2 — Application build
# =============================================================================
FROM deps AS builder

COPY . .

# Touch the stub entry points so cargo detects that sources changed.
RUN touch src/main.rs src/lib.rs \
    crates/core/src/lib.rs \
    crates/surrealdb/src/lib.rs \
    crates/telemetry/src/lib.rs

RUN cargo build --release --locked

# =============================================================================
# Stage 3 — Runtime image
# Debian Bookworm slim keeps the image small while providing glibc and
# CA certificates required for TLS connections to LLM providers.
# =============================================================================
FROM debian:bookworm-slim AS runtime

# Avoid interactive prompts from apt.
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

# Run as a non-root user.
RUN groupadd --system valymux \
 && useradd --system --gid valymux --no-create-home valymux

WORKDIR /app

COPY --from=builder /build/target/release/valymux /app/valymux

# Ensure the binary is not writable.
RUN chmod 555 /app/valymux

USER valymux

# Expose the default port. Override with SERVER_PORT at runtime.
EXPOSE 3000

# All configuration is via environment variables — see .env.example.
ENTRYPOINT ["/app/valymux"]
