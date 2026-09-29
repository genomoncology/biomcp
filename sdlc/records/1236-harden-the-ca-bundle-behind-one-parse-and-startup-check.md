---
base: 7a21702f
head: 2ea56fe1 (merged c14dcaaa with reconciliation)
---

Hardened the CA bundle behind one parse and a startup check, from the
follow-ups issue.

The resolved bundle — certificates or initialization failure — is stored
in one process-wide `OnceLock`; every builder (shared and uncached
provider clients, ORCID, ClinGen CSPEC, FDA orphan, trial documents, the
health runner) reads the cached result, and a parse counter incremented
only around real parsing proves one parse per process through
unit-test-executable reentry children. `serve`, `mcp`, and `serve-http`
validate before the stdio transport is created and before the HTTP bind:
an explicit `BIOMCP_CA_BUNDLE` failure stops the server before it
accepts a session, and a bad fallback keeps a single warning. Rotation
applies after restart, stated in the docs. Parse errors carry fixed
reasons with a one-based certificate index and never echo file content;
the mixed-fallback warning names `BIOMCP_CA_BUNDLE` as the clean-bundle
path; non-UTF-8 values keep the raw OS path for opening. Root
additivity is pinned three ways: a repository-wide source-contract test
against disabling calls, the Reqwest TLS-root feature pin, and the
private-CA handshakes — now five dedicated client tests (FDA orphan,
trial documents, ORCID, CSPEC, and the health probe, which honestly
documents that it exercises the orphan client the health runner builds,
since `health_http_client` has no endpoint override). AlphaGenome stays
documented, not wired: the gRPC client follows `SSL_CERT_FILE`/
`SSL_CERT_DIR` in place of the OS store, and bundle wiring is recorded
as future work.

Evidence: design accepted on the fourth review; code review REJECT once
(unbounded connects, misnamed health test) with the fixes re-reviewed
ACCEPT; yellow gate at 2ea56fe1 — `make lint` OK, `make test` OK,
`make spec` OK, and the TLS contract suite green three times (once in
the lane, twice as focused repeats). Four gate cycles were needed for
contract pins: the formatter shapes, the health API's canonical name,
the branch-side shell baseline, and the test-selector classification.
The merge reconciled the size inventory (shell.rs 2,230, tickets 1235
and 1236) and both env classifications.

Residuals: `health_http_client` itself has no endpoint override and no
direct TLS handshake test; it is covered by the startup and policy
tests. AlphaGenome bundle wiring is deferred with the decision recorded.
