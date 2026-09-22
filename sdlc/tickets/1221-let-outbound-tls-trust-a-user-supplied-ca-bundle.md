---
flow: build
priority: 3
deps: []
---

# 1221: Let outbound TLS trust a user-supplied CA bundle

## Goal

BioMCP works on networks that route outbound HTTPS through a corporate or internal root CA. Today the shared reqwest client trusts only the bundled webpki roots, so every upstream call fails in those environments. GitHub issue #250.

## Current Facts

- `Cargo.toml:49` builds reqwest with `rustls-tls`, which is the bundled webpki roots only: no system store and no bundle support. The reporter confirmed the failure on 0.8.25 and on main at `aafb52b` (`0.9.0-dev.6`); main is now `0.9.1-dev.1`.
- The reporter's preferred fix was switching to the OS store, and they explicitly accepted the alternative this ticket takes: "an explicit CA-bundle env var (e.g. `SSL_CERT_FILE`) read at startup and applied via `reqwest::ClientBuilder::add_root_certificate()`".
- Production client construction is not one seam. Three builders live in `src/sources/ordinary_url_policy.rs:193`, `:249` (used by vaers, gencc, gprofiler, cbioportal_download, and `sources/mod.rs:1314`), and `:280`. Four direct reqwest sites exist at `src/sources/fda_orphan.rs:508`, `:664`, `src/entities/trial/documents.rs:227`, and `src/cli/health/runner.rs:302`. AlphaGenome's gRPC client (`src/sources/alphagenome.rs:44-53`) already reads native OS roots through tonic's `tls-roots` (`Cargo.toml:59`).
- `HTTP_CLIENT` (`src/sources/mod.rs:343`, `:1136-1150`) and `HEALTH_HTTP_CLIENT` (`src/cli/health/runner.rs:296`) are `OnceLock`s, so the environment must be set before the first client build.
- `Certificate::from_pem` stores bytes without parsing; parsing happens at client build, and a bundle with no certificates adds zero roots and still succeeds. A malformed or certificate-less bundle is silently ignored.
- `danger_accept_invalid_certs` and `tls_built_in_root_certs(` appear nowhere in the repository, and no path disables verification.
- Every `BIOMCP_*` variable read in `src/` must be classified in exactly one `docs/reference/configuration.md` section (`tests/surface/test_source_configuration_docs_contract.py:148-185`). The fail-closed transport inventory (`tests/test_provider_network_policy.py:44-89`) scans `src/sources` and `src/entities`, so `src/cli/health/runner.rs` is invisible to it.
- No TLS fixture exists: `rcgen` and `tokio-rustls` are not dev-dependencies, and loopback TCP is permitted by the offline gate (`tools/check-offline-network:112-118`, `tests/test_offline_gate_contract.py:230-233`). `make test` runs `--locked` (`Makefile:29`), and a new `reqwest::Client::builder()` in a test module breaks `test_reqwest_transport_construction_has_a_fail_closed_inventory`.

## Design

- Parse the bundle in BioMCP with `Certificate::from_pem_bundle` against the builder, require at least one certificate, and add every certificate to the root store. Parsing here, not at client build, is what lets a bad bundle fail with the path named. The bundle adds to the bundled roots and never replaces them.
- Wire the helper into the three builders in `ordinary_url_policy.rs` and the four direct reqwest sites, including the health client. Leave AlphaGenome's gRPC client alone and document that it already uses OS roots.
- Precedence: `BIOMCP_CA_BUNDLE` first; a blank or whitespace-only value counts as unset, following `ordinary_url_policy.rs:207-209`. `SSL_CERT_FILE` applies only when the former is unset; a blank or unreadable `SSL_CERT_FILE` warns and continues, while a readable but malformed bundle fails naming the path. Client certificates, mTLS, proxies, and `SSL_CERT_DIR` stay out of scope.
- Fail before any request with a distinct error that carries the path, rendered in both text and `--json` output.
- Offline proof: add `rcgen` and `tokio-rustls` as dev-dependencies, generate a private CA and a `127.0.0.1` leaf per run, stand up a loopback TLS server behind the existing provider-override seam, and drive it from a subprocess test (`env!("CARGO_BIN_EXE_biomcp")`) that scrubs both variables. The untrusted case must fail the handshake with the fixture recording zero connections; missing, malformed, and certificate-less bundles must fail before any connection.
- Docs: one row for `BIOMCP_CA_BUNDLE` and the `SSL_CERT_FILE` fallback in the operator section of `docs/reference/configuration.md`; a trailing troubleshooting section that does not renumber the existing list; a mention from `docs/reference/error-codes.md:40` and the boundary paragraph in `docs/reference/data-sources.md:90-94`; and the "never disables verification" assertion beside `tests/test_provider_network_policy.py:80-89`, with the health client brought under a scan.

## Acceptance

1. With `BIOMCP_CA_BUNDLE` pointing at the fixture CA, a request to the loopback TLS fixture succeeds; with it unset and `SSL_CERT_FILE` scrubbed, the same request fails at the handshake.
2. A missing, unreadable, malformed, or certificate-less bundle fails before any request with a message naming the path, in text and `--json` output, and the fixture records zero connections.
3. No code path disables certificate verification or removes the bundled roots, and the fail-closed inventory covers every builder this ticket touches, including the health client.
4. `BIOMCP_CA_BUNDLE` and the `SSL_CERT_FILE` fallback are documented and pass the configuration docs contract.
5. `make lint`, `make test`, and `make spec` pass on yellow at the pushed SHA, with the new TLS test run three times in a row.
6. CI `canonical-gates` is green on main at the pushed SHA.

## Out of scope

- Switching the default root set to the OS store, in either the CLI or the MCP contract client.
- Proxies, mTLS, client certificates, and `SSL_CERT_DIR`.

## Review

- Design review: pending
- Code review: pending
