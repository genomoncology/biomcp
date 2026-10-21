---
base: afb32c27
head: 5e712966
---

Let an unusable `SSL_CERT_FILE` degrade to a warning instead of failing
every command, from the issue filed 2026-09-23.

The parse-failure seam in `src/sources/ca_bundle.rs` now splits by origin:
an explicit `BIOMCP_CA_BUNDLE` still fails closed with the `ca_bundle`
error naming the path, and any parse failure under the `SSL_CERT_FILE`
fallback warns on stderr and continues with the bundled roots, mirroring
the existing unreadable-fallback branch. All five confirmed bad inputs
(empty file, one bad certificate among good, leading byte-order mark,
`BEGIN TRUSTED CERTIFICATE`, DER) funnel through the single
`parse_certificates` seam, so no input is special-cased. Once-lock client
caching still resolves the bundle once per process; the health runner
inherits the policy through the same `ca_bundle::build_client` path.

This reverses the fail-closed stance ticket 1221 recorded for the
fallback. The operator who sets `BIOMCP_CA_BUNDLE` opted in and still
fails closed; a system-wide `SSL_CERT_FILE` the user never set for BioMCP
no longer breaks every command on upgrade.

The contract tests cover each bad input under both variables: explicit
exits non-zero with the path in the error and in `--json`; fallback exits
with the degrade warning asserted on stderr, a per-case connection delta
(each dropped fallback still attempts its connections), and no completed
session against the private-CA fixture. `docs/reference/configuration.md`
states the split and that a fallback bundle that cannot be parsed is
dropped whole, good certificates included, after one warning.

Evidence: yellow gate at 5e712966 — `make lint` OK, `make test` OK,
`make spec` OK; the TLS contract suite passed three times (once in
`make test`, twice as focused `binary(tls_ca_bundle_contract)` runs,
8/8 each). A first gate at 63678004 failed on a formatting nit and a
too-strict per-run connection assertion (one run may open several
connections); both were fixed in 5e712966.

Residuals: a fallback bundle with one bad certificate among good ones is
dropped whole, so the good custom roots are discarded after one warning;
documented in `configuration.md`. The warning text is asserted only for
the five broken inputs, not for the parseable-fallback success path.
