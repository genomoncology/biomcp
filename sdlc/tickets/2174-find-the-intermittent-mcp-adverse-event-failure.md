---
priority: 3
---

# 2174 — Find the intermittent MCP adverse-event failure

Status: COMPLETE.

Milestone: 0.9.3

Depends on: 2172

Reviews: revision 2992721, accept

## Outcome

The intermittent failure of `raw_and_typed_mcp_preserve_adverse_event_subset_and_full_json_contracts` has a named cause and a fix, or a recorded finding that 2172 removed it.

- Evidence: on 2026-10-17 the test failed once at `tests/rmcp_client_contract.rs:1553` with `isError` true instead of false, in a gate run on code identical to a run where it passed. 91f4afc46 now exposes the MCP error reply in the failure output. The cause is unknown. One candidate, unconfirmed: the adverse-event command reads openFDA through the HTTP cache, and on main a cache lock or I/O failure fails the source, which 2172 changes.
- Starts from: the next failure's error reply, captured by 91f4afc46's diagnostics, and the gate logs.
- Keeps: the contract the test checks.
- Changes: `sdlc/tickets/2174-find-the-intermittent-mcp-adverse-event-failure.md` for the approved repeat-run finding. If a failure recurs, claim its actual test/source files before changing them; preserve the existing diagnostics and contract.
- Proof: name the cause from a captured error reply, or reproduce it by running the test repeatedly through one build job on the build host within its configured job limit. Show the failure before the fix and its absence across the same number of repeats after. If 2172 lands first and fifty repeats show no failure, record that finding and close.
- Defers: nothing. This ticket gets no fourth gate rerun as a substitute for a cause.

Coordinator outcome before the repeat run: Check whether the unchanged adverse-event MCP contract recurs after the reviewed fail-soft cache and shared-client batch. Actual main62cefadf94b11b25bc5f79d88723aaec049dbf06 contains carrier790c32a and the companion closure. Its cache product/test inputs match the passing landing gate1792267083498-22341 on reviewed sourcedbaeb8bc7; only landing records differ. The historical once-only `isError` failure remains the failing evidence; the original cause is unconfirmed. PM explicitly permits closure if fifty repetitions after2172 show no failure. Run those fifty actual contract executions through one build job, reusing the qualified gate archive and CLI where available. Preserve complete reply/equality/count assertions and bounded existing failure diagnostics. No source or assertion change is planned. Record the exact command, artifact provenance, count and every failure; a clean result establishes non-recurrence under this check, not historical causation. The actual-main full check runs separately in parallel.

## Progress

- 2026-10-17 started

Repeat command and provenance before execution: Copy the archive and paired CLI from passing gate1792267083498-22341, sourcedbaeb8bc7149b70d2ade84fce12da7bad5e80530, into the repeat job's owned build output. The archive SHA256 is6db66630988fc7853ee9227f83fe4824f15247b7940944458e7654bd59d6028a; CLI SHA256 is04b0d81f21864fb8ff306ce9ac47b692638b1d358fe54c40d82201c2996a67f0 and its identity is0.9.3+gdbaeb8bc. That source differs from actual main62cefadf9 only in two ticket records. With canonical job-local TMPDIR and both BIOMCP_BIN and CARGO_BIN_EXE_biomcp set to the owned qualified CLI, run `tools/run-offline -- python3 tools/report-slow-tests.py cargo nextest run --archive-file "$PWD/.cache/routine-tests.tar.zst" --extract-to "$PWD/.cache/adverse-event-repeat" --workspace-remap "$PWD" --config-file "$PWD/.config/nextest.toml" -j 4 -E 'binary(rmcp_client_contract) and test(/^raw_and_typed_mcp_preserve_adverse_event_subset_and_full_json_contracts$/)' --stress-count 50 --max-fail 1 --failure-output immediate`. Actual tool help confirms stress-count support; extraction occurs once. No compilation or new assertion is planned. The wrapper has no network sandbox on this platform; the contract uses its existing fixture boundaries.

Approved repeat finding: Job1792267485428-95569 on actual main62cefadf94b11b25bc5f79d88723aaec049dbf06 exited0 after fifty actual executions, all passed with zero failures; the log includes fifty distinct PASS results and the matching50/50 iteration summary. Both the paired owned CLI and the archive-extracted CLI matched the qualified hash above. One archive extraction served all repetitions; no compilation, source change, assertion change or behavioral retry occurred. Earlier1792267458848-93381 exited96 before any test because the extraction directory was absent; creating that directory corrected setup, and its zero executions are excluded from the fifty. This meets the PM's stated closure condition after the cache batch. It establishes non-recurrence under this check, not historical causation. The existing bounded diagnostics remain available.

Prepared closure: This candidate changes only ticket records, including the already recorded observation progress for2175 and2176; their statuses remain OPEN through final rehearsal. Main62cefadf9's full check2156/1792267362041-77812 is running and must pass before this ordinary landing. The unchanged diagnostic changelog bullet already landed with the cache batch. Reuse the passing combined gate for identical product inputs; no new source or test implementation is introduced by this finding.
