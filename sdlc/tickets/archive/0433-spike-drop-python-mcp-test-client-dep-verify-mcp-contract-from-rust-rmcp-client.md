---
flow: spike
priority: 5
---
# SPIKE: drop Python mcp test-client dep, verify MCP contract from Rust (rmcp client)

That single dev dependency drags in `starlette`, `pyjwt`, and `pydantic-settings`, which is the entire source of the repo's open GitHub Dependabot alerts (7 of them, all on a test-only tree that never ships). Every few months a new advisory on that tree turns the security tab red and costs a bump ticket, even though the exposure is ~nil. If the MCP contract can be verified from Rust instead, we can delete `mcp` from the dev extras and the Python tree (and its recurring alert noise) goes with it.

Completed under March on 2026-06-22, as March ticket 433. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/433-spike-drop-python-mcp-test-client-dep-verify-mcp-contract-from-rust-rmcp-client
