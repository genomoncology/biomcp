# The server does not implement MCP revision 2026-07-28

GitHub issue #248 (filed 2026-08-24 by `hasmcp-dev`, running
`@hasmcp/mcp-spec-test` 0.1.1 — a real third-party conformance runner first
released 2026-08-23). Verified against 0.8.25 and 0.9.0-dev.5 on 2026-08-25.

What the 2026-07-28 revision requires that we lack: a mandatory
`server/discover` RPC that advertises the revisions the server can serve
(`supportedVersions`), its capabilities, its identity, and usable cache
hints — answerable before any session or handshake — plus serving a
version-less first request on a default revision.

Verified behavior of `biomcp serve` (stdio, dev build and released 0.8.25
alike):

- `initialize` with `protocolVersion: "2026-07-28"` negotiates down cleanly
  to `2025-11-25` — a correct answer under the revisions we implement.
- `server/discover` sent after a valid handshake gets a proper JSON-RPC
  `-32601` method-not-found error — well-behaved, but the method is absent.
- Any non-handshake first message — `server/discover`, a version-less
  `initialize` — makes the server print "This command expects an MCP client
  on stdin" and exit. This one behavior accounts for seven of the tool's
  eight reported failures: the discover-first probe timed out (no response
  within 10s), five further `server/discover` sub-checks then read fields
  of a result that never arrived, and the version-less request got "server
  exited."
- The remaining reported failure — "initialize must return a
  protocolVersion, got undefined" — did not reproduce here; the server
  returns a protocolVersion on direct probe. Most likely their harness saw
  the already-exited server from the earlier probe.

The tool's own 2025-11-25-revision run came back fully conformant on
everything it could verify, so clients on the revisions we implement
(2025-06-18, 2025-11-25) are unaffected today. The gap is exactly: one
spec revision behind, one mandatory method missing, and an exit where a
default-serve belongs.
