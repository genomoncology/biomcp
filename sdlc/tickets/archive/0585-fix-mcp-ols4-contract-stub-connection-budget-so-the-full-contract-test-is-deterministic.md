---
flow: quickfix
priority: 11
---
# Fix MCP OLS4 contract stub connection budget so the full-contract test is deterministic

The MCP full-contract test's local OLS4 stub caps its connection budget too low, so the contract's own traffic can exhaust it and the final call returns `is_error: true`. In `crates/biomcp-mcp-contract-client/src/lib.rs`, `start_ols4_stub()` accepts only eight connections (`listener.incoming().take(8)`), but the full contract performs three `discover BRCA1` calls whose OLS4 requests can exceed eight before the last call (fails at `lib.rs:787`). It reproduces under serial, parallel, and targeted runs — a deterministic fixture-capacity bug, not a runtime defect.

Completed under March on 2026-07-17, as March ticket 585. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/585-fix-mcp-ols4-contract-stub-connection-budget-so-the-full-contract-test-is-deterministic
