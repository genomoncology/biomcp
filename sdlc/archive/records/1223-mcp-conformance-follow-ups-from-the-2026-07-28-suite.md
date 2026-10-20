---
base: 49ac0a81
head: 73195a46
---

Closed the three remaining conformance failures from the 0.1.5 suite.

The typed `search` and `get` tool schemas now declare `"type": "object"` at the
root alongside `oneOf` (`src/mcp/shell.rs`, `src/mcp/shell/typed_get.rs`), so
stock MCP SDK clients can list tools; `variant_erepo` already had that shape.
`subscriptions/listen` acknowledges exactly the requested types the server
supports — `toolsListChanged` and `resourcesListChanged` — matching the
2026-07-28 acknowledgment contract quoted from the suite's vendored schema.
Prompts and resource subscriptions stay out because the server has neither a
prompts capability nor a resource-subscription surface. The legacy-metadata
stateless rejection (`-32022`) is pinned in `spec/surface/mcp.md` and the
protocol tests, and `scripts/release-smoke.sh` reads the entity schema from the
matching `oneOf` branch.

Verified on yellow at 73195a46: manual suite against the working-tree binary
reports `31 passed, 0 failed, 5 not verified, 1 recommended not met`, up from
28/3/5/1. `cargo fmt --check` and the hook's clippy command pass; focused
nextest ran 39/39; the focused MCP and release-smoke pytest ran 7/7; the
reaping test ran 5/5 in isolation; `make spec` passed clean on the idle host.

The full gate at e559cae2 recorded `make lint` OK and 3756/3756 Rust tests,
with one Python failure in
`test_run_wrapper_sigkill_reaps_server_group_and_owned_root` and a spec-preparation
`Text file busy`, both traced to a leftover `rmcp_streamable_http_contract`
process from an earlier interrupted run; both passed after the host was cleaned.
A later spec run also hit an unrelated `spec/entity/clingen-cspec.md` fixture
failure while another session's benchmark build saturated the gate host; it
passed on the idle host.

Code review ACCEPT with the resource-subscription rationale corrected from
"resources never update" to the absent subscription surface.
