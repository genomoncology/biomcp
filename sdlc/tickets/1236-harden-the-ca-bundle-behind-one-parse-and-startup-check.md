# Harden the CA bundle behind one parse and startup check

Filed from `sdlc/issues/2026-09-23-ca-bundle-follow-ups.md`; revised after a REJECT design review (2026-09-23). Before 0.9.1.

## Design

1. **Parse once**: a process-wide `OnceLock` in `src/sources/ca_bundle.rs`
   holds the resolved bundle — parsed certificates or the initialization
   failure — so concurrent callers cannot retry parsing, and every
   builder (`sources/mod.rs:1170-1271` shared and uncached clients,
   `orcid.rs:128-130`, `clingen_cspec.rs:42-44`, `fda_orphan.rs:511-517`,
   `trial/documents.rs:225-227`, `cli/health/runner.rs:294-302`) reads
   the stored result. The existing shared-client and health-client locks
   are untouched; client caching and `--no-cache` behavior are
   unchanged. Rotation takes effect uniformly after process restart:
   the `OnceLock` intentionally freezes the startup snapshot, and the
   docs say so.
2. **Startup validation**: `serve`, `mcp`, and `serve-http` load the
   bundle before creating the stdio transport (`src/mcp/shell.rs:1448-1460`)
   and before the HTTP bind (`src/mcp/shell/http.rs:429-461`). An
   explicit `BIOMCP_CA_BUNDLE` failure stops the server before it
   accepts a session; the fallback keeps its single warning.
3. **One-parse measurement**: `ca_bundle` keeps a process-wide parse
   counter (an atomic incremented only at real parse attempts, not cache
   reads). A test drives several builders and the load path and asserts
   the counter equals one; the mixed fallback warning appears exactly
   once across those calls.
4. **Startup tests**: binary-level tests spawn the stdio server and the
   HTTP server with an invalid explicit bundle and assert the process
   exits non-zero before accepting a session, and with a bad fallback
   assert startup succeeds with one warning.
5. **AlphaGenome stays documented, not wired**: for 0.9.1 the gRPC
   client keeps Tonic's native-roots path, which rustls-native-certs
   replaces with `SSL_CERT_FILE`/`SSL_CERT_DIR` when set.
   `docs/reference/configuration.md:64-69` and
   `docs/reference/data-sources.md:93-97` are corrected to state that
   AlphaGenome follows those variables in place of the OS store and that
   `BIOMCP_CA_BUNDLE` does not apply to it. Wiring the bundle through
   `ClientTlsConfig::ca_certificate` is recorded as future work needing
   retained PEM bytes.
6. **Error hygiene**: parse errors map to fixed reasons with a one-based
   certificate index and never echo file content — the rustls PEM
   iterator's display errors expose line bytes
   (`rustls-pki-types-1.14.0/src/pem.rs:474-510`); enumerating the
   iterator results makes index-tagged fixed reasons feasible without a
   fork. The mixed-fallback warning names `BIOMCP_CA_BUNDLE` as the
   clean-bundle path. Non-UTF-8 values keep the raw OS string for
   opening the file; only the error text is lossy.
7. **Structural root assertion**: instead of a handshake the offline
   fixture cannot issue (the `TlsFixture` CA is private), a structural
   test asserts a configured builder keeps the bundled Mozilla roots
   enabled alongside the bundle certificates.
8. **Tests**: contract additions for missing, unreadable, directory, and
   blank fallbacks; an invalid explicit bundle with a valid fallback
   fails closed; handshake tests for the health client, fda_orphan,
   trial documents, ORCID, and CSPEC; the child env pins
   `RUST_LOG=warn`. The provider-network policy test counts real
   `build_client`/`configure` calls per builder (including
   `fda_orphan.rs:668`) instead of text mentions. `fda_orphan.rs:511-520`
   distinguishes configuration errors from later build failures.

## Acceptance

- The parse counter equals one across a multi-builder test; the fallback
  warning appears once.
- Startup tests prove the stdio and HTTP servers stop on an invalid
  explicit bundle before accepting a session and start with one warning
  on a bad fallback.
- The structural test proves bundled roots stay enabled beside the
  bundle.
- All listed contract tests pass on yellow across three runs; the docs
  state the real AlphaGenome behavior.
- Full yellow gate at the head SHA; the issue file gains a Resolved
  section for the items covered; a record lands naming deferred items
  (AlphaGenome bundle wiring).

## Review

- Design review: REJECT once 2026-09-23 (gpt-5.6-sol, medium) — four
  findings folded in: AlphaGenome documented rather than wired for
  0.9.1, binary startup tests added, a parse counter defines the
  one-parse proof, and a structural root assertion replaces the
  impossible public-root handshake. Second review pending.
- Code review: pending
