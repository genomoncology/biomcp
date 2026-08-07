---
base: b586085ac032f079671ede0c08cbf17816183bd3
head: e39b236ff8d42eb7befd1ca91075b086d851f46e
---
The MCP full-contract test's local OLS4 stub caps its connection budget too low, so the contract's own traffic can exhaust it and the final call returns `is_error: true`. In `crates/biomcp-mcp-contract-client/src/lib.rs`, `start_ols4_stub()` accepts only eight connections (`listener.incoming().take(8)`), but the full contract performs three `discover BRCA1` calls whose OLS4 requests can exceed eight before the last call (fails at `lib.rs:787`). It reproduces under serial, parallel, and targeted runs — a deterministic fixture-capacity bug, not a runtime defect.

Imported from March ticket 585. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/585-fix-mcp-ols4-contract-stub-connection-budget-so-the-full-contract-test-is-deterministic
