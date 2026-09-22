---
flow: build
priority: 3
deps: []
---

# 1221: Let outbound TLS trust a user-supplied CA bundle

## Goal

BioMCP works in environments that route outbound HTTPS through a corporate or internal root CA. Today the shared reqwest client trusts only the bundled roots, so `biomcp get gene BRAF protein` and every other upstream call fail in those networks. GitHub issue #250.

## Current Facts

- The shared HTTP client is built with `reqwest` and the `rustls-tls` feature (`Cargo.toml`), so there is no user root store and no supported way to add one.
- The reporter confirmed the failure on 0.8.25 and again on `main` at `aafb52b` (`Cargo.toml` `0.9.0-dev.6`).
- No environment variable or config key currently names a CA bundle.

## Design

- Resolve a CA bundle path from configuration: `BIOMCP_CA_BUNDLE`, falling back to `SSL_CERT_FILE`.
- When set, load the PEM file and add each certificate to the rustls root store for the shared client. When unset, behavior is byte-for-byte today's.
- Never disable certificate verification and never accept an invalid bundle silently: a missing or unparsable file fails the command with the path named.
- Document the variable in the configuration and troubleshooting pages, and add an offline test that stands up a local TLS fixture signed by a private CA, proves a call succeeds with the bundle and fails without it.

## Acceptance

1. With `BIOMCP_CA_BUNDLE` pointing at the fixture CA, a request to the local TLS fixture succeeds.
2. Without it, the same request fails with a certificate error, unchanged from today.
3. A missing or malformed bundle fails with a message naming the path, and no request is attempted with verification disabled.
4. Docs name the variable and where it is read.
5. `make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- Client certificates, mTLS, and proxy configuration.
- Changing the default root set or the TLS backend.

## Review

- Design review: pending
- Code review: pending
