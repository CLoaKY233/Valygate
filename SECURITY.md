# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.x (main) | Yes |

Only the latest commit on the `main` branch receives security fixes. No backport releases are currently provided.

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Report security issues through GitHub's private [Security Advisory](https://github.com/CLoaKY233/Valymux/security/advisories/new) feature. This keeps the disclosure confidential until a fix is available.

Include the following in your report:

- A description of the vulnerability and its impact.
- The component or file(s) affected.
- Steps to reproduce or a proof-of-concept (if safe to share).
- Any suggested mitigations you have identified.

## Response Process

1. **Acknowledgement** — You will receive a confirmation within 72 hours.
2. **Assessment** — The maintainers will assess severity using [CVSS v3.1](https://www.first.org/cvss/v3.1/specification-document).
3. **Fix** — A patch will be developed in a private branch.
4. **Disclosure** — A GitHub Security Advisory and, where applicable, a CVE will be published after the fix is merged. You will be credited unless you prefer otherwise.

## Scope

The following are in scope:

- Remote code execution
- Authentication bypass or privilege escalation
- Exposure of secrets, API keys, or credentials
- Cryptographic weaknesses in `crates/surrealdb` (key derivation, encryption)
- Injection vulnerabilities in the proxy or request pipeline

The following are out of scope:

- Denial-of-service via resource exhaustion on the caller's own instance
- Vulnerabilities in third-party dependencies (report those upstream; we will bump the version once a fix is available)
- Issues that require physical access to the server

## Security-Sensitive Areas

| Area | Notes |
|------|-------|
| `crates/surrealdb/src/crypto.rs` | AES-GCM encryption, PBKDF2 key derivation for stored provider secrets |
| `src/rts/extractors.rs` | `RequireAuth` middleware — API key validation logic |
| `src/sys/config.rs` | Reads secrets from environment variables |
| `.env` / environment | Must never be committed; always use `.env.example` as the template |

## Responsible Disclosure

This project follows a coordinated disclosure model. Please allow a reasonable period (up to 90 days) for a fix to be developed before public disclosure.
