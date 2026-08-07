---
flow: spike
priority: 9
---
# Reset BioMCP test strategy around request contracts

The desired direction is a much slimmer routine gate: test that CLI commands route to the right internal request/plan objects, test that source clients construct the right HTTP requests and map fixture responses correctly, and keep only a tiny smoke layer for end-to-end confidence. Routine preflight should not call every public API or make unrelated live-source canaries block work.

Completed under March on 2026-05-22, as March ticket 371. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/371-reset-biomcp-test-strategy-around-request-contracts
