---
flow: build
priority: 10
---
# Keep provider credentials and request URLs out of logs

BioMCP currently depends on choosing `Display` rather than `Debug` at every
logging call. That is not a safe credential boundary. `BioMcpError` has a
scrubbed public display, but its derived debug form can contain a nested HTTP
error with the complete request URL and query credentials.

Federated article search logs a swallowed provider error with `?err` at warning
level. The same path is used by ordinary article search and `search all`.
Raw external-error debug fields also exist in PMC and article full-text paths.
A real CLI reproduction exposed a configured credential; do not copy that
credential into tests, tickets, fixtures, or commits.

## Logging contract

Define one structured, safe projection for an external failure. It carries only:

- provider and operation;
- a stable class such as timeout, connection, HTTP status, decode, unavailable,
  or internal;
- an HTTP status when one is safe and available; and
- a bounded scrubbed message that contains no scheme, host, path, query string,
  header value, token, or nested HTTP error debug output.

All warning, info, and debug logs for provider or transport errors use that
projection. No safety property may depend on a tracing format sigil. The user
facing `BioMcpError` display and JSON error envelope remain useful and
source-attributed.

## Done when

- A local article provider fixture puts the recognizable fake secret
  `biomcp-log-secret-DO-NOT-USE` in a query parameter and forces a transport and
  an HTTP failure.
- Default article federation and `search all` complete their normal degraded
  behavior without the secret or full request URL appearing on stdout or
  stderr.
- The same assertions pass with debug logging enabled and cover the PMC and
  full-text raw-debug sites.
- Safe provider, operation, class, and status diagnostics remain visible.
- A source audit rejects raw `?err` or `?error` tracing fields for external
  errors unless the value is the approved safe projection.
- No routine test uses a real credential or public network.

## Authorized test changes

Design commits may restate logging assertions and provider-error fixtures in
`tests/json_error_contract.rs`, `src/entities/article/search/tests.rs`,
`src/entities/article/fulltext.rs`, and `src/sources/pmc_article.rs`. They may
also extend `tests/test_quality_ratchet_contract.py` and
`tools/check-quality-ratchet.py` with the source audit. Existing public JSON
error fields and degraded-search assertions must not be weakened.

The src line ceiling may rise by at most 160 lines.
