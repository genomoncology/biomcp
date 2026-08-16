---
flow: quickfix
priority: 9
---

# Reject invalid HTTP host policies before listening

`biomcp serve-http --allowed-hosts 'bad host'` starts successfully and logs that it is listening, then answers every route with 403, including `/health` and `/readyz`, because the host policy middleware wraps the whole router. The allowlist is only trimmed of whitespace and empties when the server starts; each entry is not parsed until a request arrives, and an entry that cannot be parsed matches nothing, so a typo produces a live process that serves nothing. An operator sees a healthy startup and a dead server.

Validate and normalize every explicit allowlist entry before binding the listener, and fail with a clear argument error naming the bad entry instead of starting. An entry is an exact hostname or IP address with an optional port, IPv6 included; anything else — an empty entry, a scheme, a path, a wildcard, internal whitespace, a malformed address, or a port that is zero or out of range — is a startup error. Normalize case, trailing dots, IP spelling, and duplicates once at startup and use that one normalized policy for both the RMCP transport configuration and the router's host check, so the two cannot drift. A portless entry keeps meaning "any received port" and an entry with a port keeps meaning that port only. Loopback defaults and `--unsafe-allow-any-host`, including its existing conflict with `--allowed-hosts`, are unchanged.

Done means an invalid entry produces a nonzero exit and an actionable message on stderr, with no listening log line and nothing bound on the port, and a valid policy still serves the routes it did before to the hosts it allowed before.

Authorize implementation in `src/mcp/shell/http_server.rs` and, if argument validation belongs there, `src/cli/system/mod.rs`. Authorize restating tests in `src/mcp/shell/http_server.rs`, specifically `loopback_http_defaults_to_local_host_headers`, `non_loopback_http_requires_an_explicit_policy`, and `global_host_policy_matches_names_ports_case_and_ipv6`; and in `src/cli/system/tests.rs` and `tests/test_mcp_http_surface.py`. Update `docs/getting-started/remote-http.md`, `docs/reference/mcp-server.md`, and `docs/user-guide/cli-reference.md` where they describe the accepted entry format.

Focused tests, `make lint`, `make test`, `make spec`, and `git diff --check` must pass. No version change, queue action, tag, staging, signing, promotion, or publication belongs in this ticket.
