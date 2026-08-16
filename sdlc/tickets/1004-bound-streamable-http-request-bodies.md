---
flow: quickfix
priority: 10
---

# Bound Streamable HTTP request bodies

`serve-http` currently reads arbitrarily large `/mcp` request bodies before responding. Limit the streamed request body to 64 KiB and return HTTP 413 when either a fixed-length or chunked request exceeds that limit. Normal MCP requests, Host validation, and the lightweight probe routes must keep working.

Focused executable coverage belongs in `tests/test_mcp_http_surface.py`. The public limit may be added to the `serve-http` help and `docs/reference/mcp-server.md`; existing help and HTTP surface assertions in `src/cli/system/tests.rs` and `tests/test_mcp_http_surface.py` may be restated to include it.
