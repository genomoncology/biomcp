---
base: 5754a68b
head: 739545b1
---

`serve-http` now defaults loopback binds to local Host values and rejects
unrelated Host headers. Non-loopback binds fail before opening a listener unless
the operator supplies `--allowed-hosts` or explicitly acknowledges
`--unsafe-allow-any-host`; Clap and the runtime reject combining those choices.

The unsafe startup warning and the remote deployment documentation state that
the Host check is neither authentication nor encryption. Remote guidance now
requires authentication and TLS at a trusted proxy, gateway, or private-network
boundary and does not infer trust from forwarding headers.

Focused Rust policy/help tests, seven local HTTP process tests, 15 documentation
contracts, and no-feature Clippy with warnings denied passed. The implementation
added 92 net `src` lines against the ticket's 120-line ceiling.
