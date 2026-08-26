---
flow: build
priority: 25
---

# Implement server/discover and stop exiting on first messages

GitHub issue #248 (filed 2026-08-24). Filed from
`sdlc/issues/2026-08-25-the-server-does-not-implement-mcp-revision-2026-07-28.md`.
Amended 2026-08-26 after two refusals: the original text demanded
`initialize` behavior that contradicts the 2026-07-28 specification it
cited — that revision removes the `initialize` handshake outright (stateless
protocol; version and client capabilities ride in `_meta` on every
request). This amendment re-scopes the ticket to what is true, verifiable,
and shippable now; the full stateless adoption is ticket 1058.

## Where we stand, verified

`biomcp serve` implements MCP revisions 2025-06-18 and 2025-11-25 via the
legacy `initialize` handshake and, per the reporting tool's own run, is
fully conformant on 2025-11-25 across everything it could verify. The
2026-07-28 revision (spec tag commit `5f5440bb26a62e2cf3440b92da5a667efa03b267`,
changelog items 2-4) added a mandatory `server/discover` RPC — servers
advertise their supported versions, capabilities, and identity; clients may
call it before any other request, and on stdio it doubles as a
backward-compatibility probe, so it must be answerable with no prior
session. We do not implement it. Worse, any non-handshake first message
makes the server print "This command expects an MCP client on stdin" and
exit — one behavior that produced seven of the eight failures the issue
reports.

## Done when

- `server/discover` is implemented on the stdio serve path, shaped by the
  2026-07-28 spec text at the tag commit above: it advertises the
  `supportedVersions` the server **actually serves**, the server's
  capabilities, its identity, and usable cache hints (`CacheableResult`:
  `ttlMs`, `cacheScope`), is answerable before any session or handshake,
  and is stable across calls within its own stated TTL.
- `supportedVersions` is truthful and derived from what the server serves —
  today 2025-06-18 and 2025-11-25. Ticket 1058 adds 2026-07-28 to that
  list when and only when the server genuinely serves it; this ticket must
  not advertise a revision it does not implement.
- A non-handshake first message no longer kills the server. `server/discover`
  pre-session gets its result; any other pre-session request gets a
  conformant JSON-RPC error response. The server keeps serving the stream.
  The design reads the spec tag for the exact expected behavior of
  version-less or `_meta`-less requests and pins what the spec pins.
- The legacy `initialize` handshake for 2025-06-18 and 2025-11-25 clients
  is unchanged, byte-for-byte, with no regression in the existing handshake
  assertions.
- Offline contract tests pin the new behavior — the `make test` lane runs
  offline, so the proof is authored from the spec text, not fetched. They
  must fail on today's code for the two headline behaviors: discover
  answered pre-session; a non-handshake first message not ending the
  process.
- The MCP surface spec (`spec/surface/mcp.md`) is updated to describe
  `server/discover` and the truthful version advertisement.

## Deferred proofs

- Full stateless 2026-07-28 serving — per-request `_meta` version and
  client capabilities, `resultType` on all results, `subscriptions/listen`,
  removal of `ping`/`logging/setLevel`, `UnsupportedProtocolVersionError`
  on version mismatch — is carried by ticket 1058
  (`1058-serve-the-stateless-mcp-2026-07-28-protocol.md`), not proved here.

## Hard choices, settled

- We implement `server/discover` truthfully rather than performing the
  full revision adoption in one ticket: the honest advertisement (we serve
  two legacy revisions) plus not exiting converts the reported failures
  without a protocol overhaul bundled into the 0.9 release path.
- The third-party sweep tool remains out of every gate; the spec tag text
  and our own contract tests are what pin the behavior.
- Existing clients on the older revisions must see no behavior change;
  that constraint outranks any convenience the new revision offers.

## Out of scope

- Everything deferred to 1058. No new tools, no tool schema changes, no
  CLI surface changes beyond `serve`/`serve-http` protocol behavior, no
  registry or publication changes.
