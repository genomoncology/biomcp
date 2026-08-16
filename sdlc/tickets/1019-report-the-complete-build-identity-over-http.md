---
flow: quickfix
priority: 8
---

# Report the complete build identity over HTTP

The Streamable HTTP status route reports the package version compiled in at build time, so `GET /` answers `0.9.0-dev.4` while `biomcp --json version` answers the full build identity including the git revision. Two surfaces of the same running binary disagree about what is running, which makes a deployed server hard to identify from the outside.

Have the status route report the same build identity the CLI reports, adding the git revision and build timestamp alongside the version. The existing `name`, `transport`, and `mcp` fields keep their current names and values; this is additive, so nothing that reads the route today breaks.

Done means the identity fields returned by `GET /` are exactly equal to the corresponding fields of `biomcp --json version` for the same binary.

Authorize implementation in `src/mcp/shell/http_server.rs`. Authorize restating the test `index_handler_reports_streamable_http_surface` in `src/mcp/shell/http_server.rs`, and tests in `tests/test_mcp_http_surface.py` and `tests/test_streamable_http_demo.py`. Update the `GET /` row in `docs/getting-started/remote-http.md` to describe the added fields.

Focused tests, `make lint`, `make test`, `make spec`, and `git diff --check` must pass. No version change, queue action, tag, staging, signing, promotion, or publication belongs in this ticket.
