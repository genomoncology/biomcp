---
priority: 3
---

# 2174 — Find the intermittent MCP adverse-event failure

Status: OPEN.

Milestone: 0.9.3

Depends on: 2172

## Outcome

The intermittent failure of `raw_and_typed_mcp_preserve_adverse_event_subset_and_full_json_contracts` has a named cause and a fix, or a recorded finding that 2172 removed it.

- Evidence: on 2026-10-17 the test failed once at `tests/rmcp_client_contract.rs:1553` with `isError` true instead of false, in a gate run on code identical to a run where it passed. 91f4afc46 now exposes the MCP error reply in the failure output. The cause is unknown. One candidate, unconfirmed: the adverse-event command reads openFDA through the HTTP cache, and on main a cache lock or I/O failure fails the source, which 2172 changes.
- Starts from: the next failure's error reply, captured by 91f4afc46's diagnostics, and the gate logs.
- Keeps: the contract the test checks.
- Changes: claims `tests/rmcp_client_contract.rs` and whatever source file the named cause points to, plus `CHANGELOG.md` if behavior changes.
- Proof: name the cause from a captured error reply, or reproduce it by running the test repeatedly through one build job on the build host within its configured job limit. Show the failure before the fix and its absence across the same number of repeats after. If 2172 lands first and fifty repeats show no failure, record that finding and close.
- Defers: nothing. This ticket gets no fourth gate rerun as a substitute for a cause.
