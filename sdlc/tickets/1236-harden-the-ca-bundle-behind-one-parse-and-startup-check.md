# Harden the CA bundle behind one parse and startup check

Filed from `sdlc/issues/2026-09-23-ca-bundle-follow-ups.md` (sections "The bundle is parsed per call", "Found in the ticket 1231 review", "Parse errors echo file content", "Non-UTF-8 paths cannot be opened", "Tests", and the AlphaGenome item). Before 0.9.1.

## Design

1. **Parse once**: resolve and parse the bundle once behind a `OnceLock`
   and hand the parsed result to every builder (`provider_url_client`,
   Semantic Scholar, ORCID, ClinGen CSPEC, fda_orphan, trial documents,
   every `--no-cache` client). `serve`, `mcp`, and `serve-http` load and
   validate the bundle at startup: an explicit `BIOMCP_CA_BUNDLE` failure
   stops the server before it accepts a session; the fallback keeps the
   single warning. Certificate rotation now applies to every tool alike,
   and the warning prints once per process, which restores the
   `docs/reference/configuration.md` claim that clients are built once.
2. **AlphaGenome**: either add the bundle through
   `ClientTlsConfig::ca_certificate(...)` or correct
   `configuration.md:66` and `data-sources.md:95` to state that
   AlphaGenome gRPC follows `SSL_CERT_FILE`/`SSL_CERT_DIR` through
   rustls-native-certs. Record the decision.
3. **Error hygiene**: parse errors map to fixed reasons with a
   certificate index and never echo file content; a mixed fallback warns
   that the whole bundle was dropped and names
   `BIOMCP_CA_BUNDLE` as the clean-bundle path; non-UTF-8 values keep the
   raw OS string for opening the file and only the error text is lossy.
4. **Tests**: contract suite additions for missing, unreadable,
   directory, and blank fallbacks; an invalid explicit bundle with a
   valid fallback fails closed; handshake tests for the health client,
   fda_orphan, trial documents, ORCID, and CSPEC; bundled roots stay
   trusted alongside the bundle. The child env pins `RUST_LOG=warn` so
   `tests/tls_ca_bundle_contract.rs:78-84` no longer inherits the host
   setting. The provider-network policy test counts real `build_client`
   calls per builder instead of text mentions.
5. `fda_orphan.rs:517` keeps a client-build failure distinguishable from
   `unavailable()` when a bundle is configured.

## Acceptance

- One parse per process proven by tests; server startup validates the
  bundle; the warning appears once.
- All listed test additions pass on yellow across three contract runs.
- Docs state the real AlphaGenome behavior.
- Full yellow gate at the head SHA; the issue file gains a Resolved
  section for the items covered; a record lands with any deferred items
  named.

## Review

- Design review: pending
- Code review: pending
