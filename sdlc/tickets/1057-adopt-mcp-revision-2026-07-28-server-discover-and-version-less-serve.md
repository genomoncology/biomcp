---
flow: build
priority: 25
---

# Adopt MCP revision 2026-07-28: server/discover and version-less serve

GitHub issue #248 (filed 2026-08-24). Filed from
`sdlc/issues/2026-08-25-the-server-does-not-implement-mcp-revision-2026-07-28.md`,
which carries the verified behavior and the source links.

## Where we stand, verified

`biomcp serve` implements MCP revisions 2025-06-18 and 2025-11-25 and, per
the reporting tool's own run, is fully conformant on 2025-11-25 across
everything it could verify. The 2026-07-28 revision added a mandatory
`server/discover` RPC (advertise `supportedVersions`, capabilities, server
identity, usable cache hints; answerable before any session or handshake)
and requires that a version-less first request be served on a default
revision. We do neither, and any non-handshake first message makes the
server exit with "This command expects an MCP client on stdin" — one
behavior that produced seven of the eight failures the issue reports. This
ticket moves the server onto the new revision while keeping the two older
negotiations working exactly as they do today.

## Done when

- `server/discover` is implemented for the stdio serve path: it answers
  with the server's supported revisions, capabilities, and identity, is
  answerable before any session or handshake, carries usable cache hints,
  and is stable across calls within its own stated TTL. The design derives
  the exact response shape from the 2026-07-28 specification text — the
  spec, not the third-party report, is the source of truth — and the
  same derivation decides how the method behaves on `serve-http`, which
  this ticket brings along rather than leaving to drift.
- A version-less first request is served on a default revision instead of
  exiting. The default chosen is recorded in the design with its reason.
- An `initialize` requesting 2026-07-28 now settles on 2026-07-28 rather
  than negotiating down; requests for 2025-06-18 and 2025-11-25 keep
  settling exactly as they do today, with no regression in the existing
  handshake assertions.
- Whatever else the 2026-07-28 revision makes mandatory for a server of
  our shape (the reporting tool enumerates result-envelope, cache-hint,
  and subscription-listen requirements among its unverified cases) is
  decided by the design stage against the spec text: each requirement is
  either implemented in this ticket or named in a `## Deferred proofs`
  section with a successor ticket that carries it. Nothing is silently
  skipped.
- Conformance is pinned by offline contract tests against the spec's wire
  shapes — the `make test` lane runs offline, so the proof must be
  authored, not fetched. The tests must fail on today's code for the two
  headline behaviors (discover answered pre-session; version-less request
  served).
- The MCP surface spec (`spec/surface/mcp.md`) and the version-adoption
  notes it references are updated to name 2026-07-28 as a supported
  revision.

## Hard choices, settled

- We adopt the revision rather than replying "unsupported": the cost is
  one method plus a default-serve, and it removes a "not conformant"
  headline before it hardens.
- The third-party sweep tool (`@hasmcp/mcp-spec-test`) is not wired into
  any gate. It pointed at the gap; the spec text and our own contract
  tests are what pin it.
- Existing clients on the older revisions must see no behavior change;
  that constraint outranks any convenience the new revision offers.

## Out of scope

- No new tools, no changes to tool schemas or descriptions, no changes to
  the CLI surface beyond `serve`/`serve-http` protocol behavior, no
  registry or publication changes. The changelog entry for this work is
  carried by the release-notes path, not this ticket.
