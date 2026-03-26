# Contributing to ValyMux

Thank you for considering a contribution to ValyMux. This document explains how to get started, what standards the project holds, and how the review process works.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Before You Start](#before-you-start)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Making Changes](#making-changes)
- [Commit Messages](#commit-messages)
- [Pull Requests](#pull-requests)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Features](#suggesting-features)
- [License](#license)

---

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating you agree to abide by its terms.

---

## Before You Start

- Check the [open issues](https://github.com/CLoaKY233/Valymux/issues) and [pull requests](https://github.com/CLoaKY233/Valymux/pulls) to avoid duplicate work.
- For significant changes, open an issue first and discuss the approach before writing code.
- For small bug fixes or documentation improvements, a PR without prior discussion is fine.

---

## Development Setup

### Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Rust | stable (MSRV 1.85) | Install via [rustup](https://rustup.rs/) |
| cargo-audit | latest | `cargo install cargo-audit` |
| cargo-deny | latest | `cargo install cargo-deny` |

### Getting the Code

```sh
git clone https://github.com/CLoaKY233/Valymux.git
cd Valymux
cp .env.example .env
# Edit .env with your local values
```

### Environment Variables

Copy `.env.example` to `.env` and fill in the required values. At minimum you need a SurrealDB instance reachable at `SURREAL_URL` and a 32-byte hex encryption key at `SURREAL_ENCRYPTION_KEY`.

### Building

```sh
# Debug build
cargo build

# Release build
cargo build --release

# Run the server
cargo run
```

### Running Tests

```sh
cargo test
```

### Linting and Formatting

The CI pipeline enforces these checks. Run them locally before pushing:

```sh
# Check formatting (does not modify files)
cargo fmt --all -- --check

# Apply formatting
cargo fmt --all

# Run Clippy (warnings are treated as errors in CI)
cargo clippy --all-targets --all-features -- -D warnings

# Check for known security advisories
cargo audit
```

---

## Project Structure

```
.
├── src/                  # Main binary crate
│   ├── main.rs           # Entry point, server startup, graceful shutdown
│   ├── sys/              # System-level concerns (config, state, init)
│   └── rts/              # Runtime services and Axum route handlers
│       └── v1/           # OpenAI-compatible v1 API handlers
├── crates/
│   ├── core/             # Shared error types (AppError, IntoResponse)
│   ├── surrealdb/        # SurrealDB client, models, schema, crypto
│   └── telemetry/        # Tracing initialisation (JSON / pretty / compact)
└── docs/                 # Architecture and API documentation
```

Each crate in `crates/` is independently versioned. Changes to shared crates require updating dependent crates in the same PR.

---

## Making Changes

1. Fork the repository and create a branch from `main`:
   ```sh
   git checkout -b fix/short-description
   # or
   git checkout -b feat/short-description
   ```

2. Write your code following the conventions below.

3. Add or update tests for any logic you touch.

4. Run the full check suite locally (see [Linting and Formatting](#linting-and-formatting)).

5. Push your branch and open a pull request.

### Coding Conventions

- **Errors:** Use `AppError` from `valymux-core` for errors that reach HTTP responses. Use `anyhow::Error` for internal, non-user-facing errors.
- **Async:** All I/O must be async. Do not use blocking calls inside async contexts.
- **State:** Shared state goes in `Arc<AppState>`. Do not use global statics.
- **Tracing:** Use `tracing` macros (`info!`, `warn!`, `error!`, `debug!`) rather than `println!` or `eprintln!`. Include structured fields where relevant.
- **Unsafe:** Avoid `unsafe` unless there is a documented, justified reason. All `unsafe` blocks must have a `// SAFETY:` comment explaining the invariant being upheld.
- **Dependencies:** Prefer workspace-level dependency declarations in the root `Cargo.toml` over per-crate pinning. Justify new dependencies in the PR description.

---

## Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification:

```
<type>(<optional scope>): <short summary>

<optional body>

<optional footer>
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `perf`, `ci`.

Examples:

```
feat(proxy): add Anthropic streaming support
fix(surrealdb): handle connection timeout on startup
docs: add architecture overview
chore(deps): bump reqwest to 0.13.3
```

- The summary line must be 72 characters or fewer.
- Use the imperative mood ("add", not "added" or "adds").
- Reference issues with `Closes #123` or `Fixes #123` in the footer.

---

## Pull Requests

- Target the `main` branch.
- Fill in the pull request template completely.
- Keep PRs focused. One logical change per PR.
- All CI checks must pass before review.
- At least one maintainer approval is required to merge.
- Squash-merge is used; your individual commits will be squashed into one.

---

## Reporting Bugs

Use the [Bug Report](https://github.com/CLoaKY233/Valymux/issues/new?template=bug_report.yml) issue template. Include:

- The version or commit hash you are running.
- The operating system and Rust toolchain version.
- A minimal reproduction case.
- The observed vs. expected behaviour.
- Relevant log output (redact any credentials or keys).

---

## Suggesting Features

Use the [Feature Request](https://github.com/CLoaKY233/Valymux/issues/new?template=feature_request.yml) template. Explain the problem you are trying to solve, not just the solution you have in mind.

---

## License

By submitting a contribution you agree that your work will be licensed under the [GNU Affero General Public License v3.0](LICENSE) that covers this project.
