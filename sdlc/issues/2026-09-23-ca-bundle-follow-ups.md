# CA bundle follow-ups

Filed 2026-09-23 from an independent review of `v0.9.0..f2549676`. Covers ticket 1221 gaps beyond the `SSL_CERT_FILE` regression.

## Non-UTF-8 paths cannot be opened

`src/sources/ca_bundle.rs:118-131` converts the value with `to_string_lossy()` and opens the converted path. A real file named `n\xffu.pem` fails with "No such file or directory". The 1221 record says only the error text is affected. Build the path from the raw OS string and trim only valid UTF-8.

## AlphaGenome ignores BIOMCP_CA_BUNDLE

`src/sources/alphagenome.rs:50` is on by default. Its roots come through tonic `tls-roots` and rustls-native-certs 0.8.3. That library reads `SSL_CERT_FILE` and `SSL_CERT_DIR` in place of the OS store. `docs/reference/configuration.md:66` and `docs/reference/data-sources.md:95` describe it wrongly. Add the bundle through `ClientTlsConfig::ca_certificate(...)`, or document the real behavior.

## The bundle is parsed per call

Only the shared cached client and the health client are built once. `provider_url_client`, Semantic Scholar, ORCID, ClinGen CSPEC, fda_orphan, trial documents, and every `--no-cache` client re-read and re-parse the file on each call. A certificate rotation applies to some tools and not others, and the fallback warning repeats. `serve`, `mcp`, and `serve-http` never check the bundle at startup, so a bad bundle starts a server that fails every call. Parse once behind a `OnceLock`, load it at server startup, and hand it to every builder.

## Parse errors echo file content

`ca_bundle.rs:141` passes the library message through. A line `-----BEGIN SECRET token=hunter2` came back as a byte array containing the token. `docs/reference/error-codes.md:15-18` promises only the path. "invalid peer certificate: BadEncoding" describes a bundle entry as a peer. Map parse errors to fixed reasons with a certificate index.

## Tests

- `tests/test_provider_network_policy.py:129-140` counts `ca_bundle::` text. Deleting the `build_client` call at `src/sources/fda_orphan.rs:668` still passes. It also skips `clingen_cspec`, `orcid`, and `sources/mod.rs`. Match real calls with an exact count per builder.
- Only the openFDA shared client has a real handshake test. Add handshake tests for the health client, fda_orphan, trial documents, ORCID, CSPEC, and the `SSL_CERT_FILE` path, plus one proving the bundled roots stay trusted alongside the bundle.
- `fda_orphan.rs:517` turns a client build failure into `unavailable()` even when a bundle is loaded.

## Found in the ticket 1231 review

- The fallback warning prints on every client build, so once per MCP tool call. `docs/reference/configuration.md:68` says the clients "are built once". The `OnceLock` fix above resolves both; until then correct the docs line.
- The warning at `src/sources/ca_bundle.rs:115` drops a whole mixed bundle, corporate root included. Tell the user to set `BIOMCP_CA_BUNDLE` to a clean bundle.
- The missing-file fallback has no test. Missing, unreadable, directory, and blank fallbacks are tested only in the loader. No test covers an invalid `BIOMCP_CA_BUNDLE` with a valid `SSL_CERT_FILE`, which must fail closed. Add them to `broken_fallback_bundles_warn_and_continue` and the contract suite.
- `tests/tls_ca_bundle_contract.rs:78-84` inherits `RUST_LOG`. With `RUST_LOG=error` the warning is hidden and the test fails. Set `RUST_LOG=warn` on the child.
