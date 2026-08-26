---
flow: build
priority: 20
deps: ["1057"]
---

# Serve the stateless MCP 2026-07-28 protocol

Successor to 1057 (whose `## Deferred proofs` names this ticket). GitHub
issue #248 reported non-conformance with the 2026-07-28 revision; 1057
implements `server/discover` with a truthful version advertisement; this
ticket makes the server actually serve the 2026-07-28 revision and add it
to that advertisement.

The source of truth is the official 2026-07-28 specification tag, commit
`5f5440bb26a62e2cf3440b92da5a667efa03b267` — read the tag, not summaries.
What that revision makes different from the legacy handshake model we serve
today, as its changelog states: the protocol is stateless — no
`initialize`/`notifications/initialized` handshake; every request carries
its protocol version and client capabilities in `_meta`
(`io.modelcontextprotocol/protocolVersion`,
`io.modelcontextprotocol/clientCapabilities`), clients SHOULD identify
themselves per request, servers SHOULD identify themselves in each result's
`_meta`, and version mismatches return `UnsupportedProtocolVersionError`;
all results carry a required `resultType`; the list endpoints return
`CacheableResult` with `ttlMs` and `cacheScope`; `resources/subscribe` is
replaced by `subscriptions/listen`; `ping` and `logging/setLevel` are
removed.

## Done when

- A 2026-07-28 client can use the server end to end without any handshake:
  requests carry version and capabilities in `_meta`, and the server
  enforces the spec's requirements for missing or mismatched fields (the
  spec's error model, including `-32602` for missing required fields and
  `UnsupportedProtocolVersionError` for version mismatch).
- Results conform to the revision: required `resultType` on all results,
  `CacheableResult` cache hints on the list endpoints, server identity in
  result `_meta`.
- `server/discover`'s `supportedVersions` now includes 2026-07-28 —
  advertised when, and only when, the serving above is real and pinned by
  tests.
- `subscriptions/listen` is implemented per the revision for the
  notification types our server actually emits; `ping` and
  `logging/setLevel` behave as the revision prescribes for a modern
  client.
- Legacy clients on 2025-06-18 and 2025-11-25 keep the exact handshake
  behavior they have today; the design decides the detection rule (per the
  spec's guidance on legacy requests) and pins it.
- Offline contract tests derived from the spec tag cover each bullet and
  fail on today's code. The MCP surface spec (`spec/surface/mcp.md`) is
  updated to name 2026-07-28 as a served revision.

## Hard choices, settled

- The design stage reads the spec tag in full and enumerates every
  requirement applicable to a server of our shape; each is either
  implemented or named in a `## Deferred proofs` section with a successor
  ticket. Nothing is silently skipped.
- Multi Round-Trip Requests and the tasks extension are client-facing
  optional machinery; the design decides against implementing them unless
  the spec makes them mandatory for a server that never sends
  `input_required` — and records that reasoning.
- The third-party sweep tool stays out of the gates.

## Out of scope

- No new tools, no tool schema changes, no changes beyond protocol
  behavior on `serve`/`serve-http`, no registry or publication changes.
