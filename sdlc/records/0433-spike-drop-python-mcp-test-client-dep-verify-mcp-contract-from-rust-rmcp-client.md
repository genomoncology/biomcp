---
base: 7733006ac948325e79e18bb5690caf0f13648919
head: 32d6bd90a247ccee0b7b22dd49f19aeed7e4d2eb
---
That single dev dependency drags in `starlette`, `pyjwt`, and `pydantic-settings`, which is the entire source of the repo's open GitHub Dependabot alerts (7 of them, all on a test-only tree that never ships). Every few months a new advisory on that tree turns the security tab red and costs a bump ticket, even though the exposure is ~nil. If the MCP contract can be verified from Rust instead, we can delete `mcp` from the dev extras and the Python tree (and its recurring alert noise) goes with it.

Imported from March ticket 433. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/433-spike-drop-python-mcp-test-client-dep-verify-mcp-contract-from-rust-rmcp-client
