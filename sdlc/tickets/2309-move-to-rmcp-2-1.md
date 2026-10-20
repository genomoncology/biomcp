---
priority: 1
---

# 2309 — Move to rmcp 2.1

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 4d19476, accept

## Outcome

Changes: `Cargo.toml`, `Cargo.lock`, `crates/biomcp-mcp-contract-client/Cargo.toml`, `src/mcp/shell/http_server.rs`, `src/mcp/shell.rs`, `src/mcp/shell/modern.rs`, `crates/biomcp-mcp-contract-client/src/lib.rs`, `examples/rmcp_streamable_http_contract.rs`, `src/mcp/shell/http_server_security_tests.rs`, `CHANGELOG.md`, `sdlc/records/2309-mcp-security-upgrade.md`.

BioMCP and `crates/biomcp-mcp-contract-client` use rmcp 2.1.0 or later, clearing eight open GitHub security alerts. The most serious for users is the unauthenticated session-table leak in rmcp's streamable HTTP server transport, a remote denial of service against `biomcp serve-http` (`src/mcp/shell/http_server.rs`). The others cover OAuth resource metadata validation and fetching, and custom headers leaking to cross-origin redirects, mostly on the client side. MCP behavior over stdio and HTTP stays the same for clients.

- Proof: both manifests and `Cargo.lock` name rmcp 2.1 or later; the existing MCP stdio and HTTP contract suites, the contract client checks and the installed-wheel smoke pass unchanged; an outside-in test sends repeated rejected non-initializing first HTTP POSTs and proves they retain no sessions, with valid initialization and DELETE as the positive control; after landing, GitHub shows alerts 81 to 86, 90 and 92 closed.

## Evidence

GitHub Dependabot listed rmcp 1.7.0 (declared `1.1.1`) with alerts 81 to 86, 90 and 92, opened between 16 September and early October; the fixes are in rmcp 2.0.0 and 2.1.0. `cargo deny` did not report them, because these advisories are published on GitHub and not in the RustSec database the gate reads. A major version may change APIs; keep the migration local to the MCP shell and the contract client, and change no tool schemas.
