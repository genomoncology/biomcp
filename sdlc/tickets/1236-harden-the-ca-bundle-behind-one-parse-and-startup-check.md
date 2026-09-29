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
   and before the HTTP bind (`src/mcp/shell/http_server.rs:429-462`). An
   explicit `BIOMCP_CA_BUNDLE` failure stops the server before it
   accepts a session; the fallback keeps its single warning.
3. **One-parse measurement and test isolation**: `ca_bundle` keeps a
   process-wide parse counter (an atomic incremented only around real
   certificate parsing, not cache reads). Because the `OnceLock` freezes
   the first resolution for the process life, the environment-dependent
   loader tests (`ca_bundle.rs:188-350`) and the multi-builder one-parse
   measurement run in fresh child processes of the unit-test executable
   itself — the `current_exe()` reentry pattern with env-selected child
   cases, as `src/entities/gene/gencc/tests.rs:606` already does — so a
   child case can invoke the private `load()` path and read the private
   counter. The `CARGO_BIN_EXE_biomcp` harness is reserved for public
   behavior and the startup tests. One child drives several builders and
   asserts the counter equals one with a single fallback warning. No
   production state seam is added.
4. **Startup tests, all four combinations**: binary tests through the
   existing harness spawn each transport and assert — stdio with an
   invalid explicit bundle exits non-zero before accepting a session;
   stdio with a bad fallback starts and logs one warning; HTTP with an
   invalid explicit bundle exits non-zero before binding; HTTP with a
   bad fallback binds and logs one warning. Both stdio cases hold stdin
   open and assert the expected bundle diagnostic on stderr, so EOF
   guidance cannot satisfy a failure assertion; the HTTP success case
   polls readiness. Every poll, wait, and termination carries a bounded
   deadline with forced child cleanup, so no case can hang.
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
7. **Root additivity by dependency contract**: Reqwest's builder hides
   its effective root configuration, and `configure` accepts an
   already-configured builder (`ca_bundle.rs:25-36`), so pinning
   `ca_bundle` alone cannot prove retention. The proof is threefold: a
   source-contract test asserting repository-wide absence of
   root-disabling or verification-disabling builder calls; a pin on the
   Reqwest TLS-root feature in `Cargo.toml:49`; and the private-CA
   handshake tests proving the bundle participates alongside the
   default roots.
8. **Tests**: contract additions for missing, unreadable, directory, and
   blank fallbacks; an invalid explicit bundle with a valid fallback
   fails closed; handshake tests for the health client, fda_orphan,
   trial documents, ORCID, and CSPEC; the child env pins
   `RUST_LOG=warn`. The provider-network policy test counts real
   `build_client`/`configure` calls per builder (including
   `fda_orphan.rs:668`) instead of text mentions. `fda_orphan.rs:511-520`
   distinguishes configuration errors from later build failures.

## Acceptance

- The one-parse counter equals one in a child process driving several
  builders; the fallback warning appears once; the loader cases run in
  fresh children.
- Startup tests prove all four combinations (stdio and HTTP × invalid
  explicit and bad fallback).
- The source-contract test pins additive root handling in `ca_bundle`,
  and the private-CA handshakes prove bundle participation.
- All listed contract tests pass on yellow across three runs; the docs
  state the real AlphaGenome behavior.
- Full yellow gate at the head SHA; the issue file gains a Resolved
  section for the items covered; a record lands naming deferred items
  (AlphaGenome bundle wiring).

## Review

- Design review: REJECT three times 2026-09-23 (gpt-5.6-sol, medium).
  First: AlphaGenome documented rather than wired, binary startup tests,
  a parse counter, structural root assertion. Second: child-process
  isolation for loader tests, source-contract over builder introspection,
  four startup combinations. Third: the one-parse child is the
  unit-test executable via `current_exe()` reentry (not the production
  binary), the root pin covers repository-wide disabling calls plus the
  Reqwest feature, both stdio cases hold stdin and assert the
  diagnostic, and every wait is deadline-bounded with forced cleanup.
  Fourth: ACCEPT 2026-09-23 — one P2 citation fix applied here.
- Code review: REJECT once, fixes applied, re-review ACCEPT 2026-09-23
  (gpt-5.6-sol, medium)
- Verification: yellow gate at 2ea56fe1 lint/test/spec OK; TLS contract
  3x green; see
  `sdlc/records/1236-harden-the-ca-bundle-behind-one-parse-and-startup-check.md`
