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
- Changes: `sdlc/tickets/2174-find-the-intermittent-mcp-adverse-event-failure.md` for the approved repeat-run finding. If a failure recurs, claim its actual test/source files before changing them; preserve the existing diagnostics and contract.
- Proof: name the cause from a captured error reply, or reproduce it by running the test repeatedly through one build job on the build host within its configured job limit. Show the failure before the fix and its absence across the same number of repeats after. If 2172 lands first and fifty repeats show no failure, record that finding and close.
- Defers: nothing. This ticket gets no fourth gate rerun as a substitute for a cause.

Coordinator outcome before the repeat run: Check whether the unchanged adverse-event MCP contract recurs after the reviewed fail-soft cache and shared-client batch. Actual main62cefadf94b11b25bc5f79d88723aaec049dbf06 contains carrier790c32a and the companion closure. Its cache product/test inputs match the passing landing gate1792267083498-22341 on reviewed sourcedbaeb8bc7; only landing records differ. The historical once-only `isError` failure remains the failing evidence; the original cause is unconfirmed. PM explicitly permits closure if fifty repetitions after2172 show no failure. Run those fifty actual contract executions through one build job, reusing the qualified gate archive and CLI where available. Preserve complete reply/equality/count assertions and bounded existing failure diagnostics. No source or assertion change is planned. Record the exact command, artifact provenance, count and every failure; a clean result establishes non-recurrence under this check, not historical causation. The actual-main full check runs separately in parallel.
