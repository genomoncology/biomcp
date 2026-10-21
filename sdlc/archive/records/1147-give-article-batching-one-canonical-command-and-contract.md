---
flow: build
priority: 8
---

# Give article batching one canonical command and contract

BioMCP now teaches `biomcp batch article <ids>` as the canonical article
batch command. Its explicit `--mode detail|compact` switch defaults to the
ordinary article-detail projection, while the existing `biomcp article batch`
route remains a byte-compatible compact escape for existing callers except
for the deliberate 512-byte per-input safety limit.

Both modes retain the ordered mixed-success settlement envelope and now keep
at most ten item futures live. Preflight rejects invalid mode, section, count,
source, and length combinations before provider or cache work. Dropping a
batch drops queued and active item futures, including provider retry backoff,
without a later request, cache publication, detached task, or retained client.
Raw stdio and Streamable HTTP MCP expose the same CLI behavior through one
sanitized text content item and do not add a typed batch tool.

Documentation, help, list output, packaged skills, next-command suggestions,
and executable article contracts now use the canonical grammar. One migration
note documents the compatibility route without recommending or generating it.

## Evidence

Independent SOL design review accepted the corrected public grammar,
compatibility boundary, failure matrix, and cancellation proof. A first
independent code review rejected a lower-level retry-cancellation test because
it did not cross the command boundary. Remediation added a deterministic test
through the real parsed `handle_batch` command and provider middleware; a fresh
independent SOL review accepted it with no findings.

Focused tests prove the ten-future ceiling, stable ordering and duplicates,
continued settlement after item failure, command-drop cancellation, exact
compatibility bytes, compact and detail rendering, zero-request preflight,
raw stdio and HTTP MCP shape, package size, and source-package independence.
On the accepted candidate, `make lint`, `make test`, and `make spec` passed
under Node 22. The shared development host used eight nextest workers for the
test gate; hosted CI retains its normal concurrency.

## Boundary

This change does not add provider bulk APIs, deduplicate IDs, alter provider
selection, retries, timeouts, rate limits, or cache keys, add a typed MCP batch
tool, or introduce coupling to any separate 1.0 data project.
