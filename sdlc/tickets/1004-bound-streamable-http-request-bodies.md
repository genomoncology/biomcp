---
flow: quickfix
priority: 10
---

# Bound Streamable HTTP request bodies

`serve-http` currently reads arbitrarily large `/mcp` request bodies before responding. Limit each `POST /mcp` encoded body to 64 KiB, meaning 65,536 bytes inclusive. Reject a declared larger `Content-Length` before reading any body bytes. For unknown-length and chunked requests, BioMCP-owned middleware must stop after the limit is exceeded, map only that exhaustion to HTTP 413, and reconstruct bodies at or below the limit for RMCP. Do not rely on a middleware path that turns streamed exhaustion into RMCP's HTTP 500 response.

Host validation must run before body inspection: a forbidden Host with an oversized body returns 403 without the body being consumed. The limiter applies only to `POST /mcp`; `/`, `/health`, `/readyz`, and MCP GET behavior stay outside it. Normal MCP traffic and unrelated requests must continue during and after an oversized or slowly streamed request.

Focused executable coverage in `tests/test_mcp_http_surface.py` must prove a real valid MCP initialize request below the limit, an accepted request exactly 65,536 bytes, 65,537-byte fixed and chunked requests returning 413, early rejection from oversized `Content-Length` before the body is sent, forbidden-Host precedence, and concurrent normal/probe traffic. The 64 KiB per-request limit is public behavior and must appear in `serve-http --help` and `docs/reference/mcp-server.md` with matching assertions.

Implementation and help changes are authorized in `src/mcp/shell.rs` and `src/cli/system/mod.rs`; existing assertions in `src/cli/system/tests.rs` may be restated. Prefer the existing Axum body APIs. If a new direct dependency is genuinely necessary, `Cargo.toml` and `Cargo.lock` are authorized, but the implementation must still distinguish body exhaustion from unrelated read errors.
