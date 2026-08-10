---
flow: build
priority: 10
deps: ["0951"]
---
# Keep provider credentials and request URLs out of logs

BioMCP's public `BioMcpError` display already scrubs major HTTP/provider
failures, so this is not a claim that every current `%err` site leaks. The
confirmed boundary failure is raw debug projection: the derived debug form can
contain a nested HTTP error with the complete request URL and query
credentials. Relying on every future call site to choose a safe formatter is
still too fragile for a credential boundary.

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
- a scrubbed message of at most 512 UTF-8 bytes that contains no scheme, host,
  path, query string, header value, token, or nested HTTP error debug output.

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
- A source audit rejects raw `Debug` projection of external errors and any
  `Display` projection whose error type is not ratcheted to the approved safe
  public projection. Safe `BioMcpError::Display` use may remain. The audit is
  based on the value's source/type and approved helper, not a short
  variable-name list or one tracing sigil; it treats the broader type rule as
  defense in depth rather than evidence that every existing display site leaks.
- Exact-boundary and boundary-plus-one fixtures prove the scrubbed message is
  at most 512 UTF-8 bytes and ends at a valid character boundary.
- No routine test uses a real credential or public network.

## Authorized test changes

Design commits may restate logging assertions and provider-error fixtures in
`tests/json_error_contract.rs`, `src/entities/article/search/tests.rs`,
`src/entities/article/fulltext.rs`, `src/entities/article/batch.rs`,
`src/entities/pathway.rs`, and `src/sources/pmc_article.rs`. They may
also extend `tests/test_quality_ratchet_contract.py` and
`tools/check-quality-ratchet.py` with the source audit. Existing public JSON
error fields and degraded-search assertions must not be weakened.

The src line ceiling may rise by at most 160 lines.
