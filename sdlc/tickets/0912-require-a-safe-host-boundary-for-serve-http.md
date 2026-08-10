---
flow: build
priority: 10
---
# Require a safe Host boundary for serve-http

An empty `--allowed-hosts` value currently disables rmcp's local Host guard, so
a loopback server accepts `Host: attacker.example`. The docs call that open
default intentional and recommend non-loopback binding without establishing an
authentication boundary. A Host allowlist is not authentication, but accepting
every Host by default is still the wrong local and remote boundary.

## Command contract

- A loopback bind with no flags accepts `localhost`, `127.0.0.1`, and `[::1]`,
  with or without the bound port, and rejects every unrelated Host value.
- A non-loopback bind fails before opening a listener unless the operator gives
  at least one `--allowed-hosts` value or
  `--unsafe-allow-any-host`.
- `--allowed-hosts` and `--unsafe-allow-any-host` are mutually exclusive.
- The unsafe flag is the only way to accept arbitrary Host values. Help and the
  startup log state that it removes only the Host check and adds no
  authentication or encryption.
- Forwarded Host headers are trusted only when the explicit allowlist contains
  the value BioMCP actually receives. BioMCP does not infer trust from generic
  forwarding headers.

Remote documentation requires TLS and authentication at a trusted reverse
proxy or private-network boundary. It must not describe a Host allowlist as an
authentication control.

## Done when

Local process tests cover loopback defaults, port forms, an unrelated Host,
explicit allowlisting, the unsafe acknowledgement, conflicting flags, and a
non-loopback bind without either acknowledgement. No test exposes a public
listener or uses public network.

## Authorized test changes

Design commits may replace the current open-by-default assertions in
`src/cli/system/tests.rs`, `tests/test_mcp_http_surface.py`, and
`spec/surface/mcp.md`. They may restate the Host and remote-boundary claims in
`docs/reference/mcp-server.md`, `docs/getting-started/remote-http.md`,
`architecture/ux/cli-reference.md`, and their existing documentation contract
tests. Existing `/mcp`, health, readiness, and dynamic-port behavior remains.

The src line ceiling may rise by at most 120 lines.
