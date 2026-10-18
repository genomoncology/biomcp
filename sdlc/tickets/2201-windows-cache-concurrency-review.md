---
priority: 2
---

# 2201 — Review Windows cache concurrency

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 87a1073, accept

Reviews: revision 1fe4099, accept

Reviews: revision f72c011, accept

## Outcome

An agent test review on Windows covers parallel CLI runs, an MCP server and `cache clear` sharing one cache, and the size marker, under Windows' mandatory locks.

- Evidence: the 0.9.3 release review ran on Linux only; 2170's Windows CI case covers lock contention but not the full concurrent matrix.
- Changes: `tests/windows_stdio_contract.rs` `tests/test_ci_workflow_contract.py` `.github/workflows/ci.yml` `CHANGELOG.md` `sdlc/records/2201-windows-cache-concurrency-review.md`. Extend the existing portable threaded stdio fixture and manual Windows-only check; no automatic triggers or product patches.
- Proof: a recorded run on Windows with results; file defects as tickets.

## Progress

- 2026-10-18 started

PM decision: Ian approved a bounded manual GitHub Windows run for this concurrency matrix; no Windows host exists. Keep dispatch manual and scoped. Pre-code plan: reuse the existing recorded MyGene BRAF reply and Windows threaded StdioMcp; cover simultaneous CLI and persistent-MCP cache writes, warm reuse, cache clear while the MCP process remains alive, post-clear refill/reuse, the valid byte marker and unrelated sibling preservation. Use request-arrival barriers to prove overlap without inventing an ordering contract that can deadlock against legitimate operation exclusion. Preserve the approved conservative-overcount bound until maintenance. Report defects separately. Code preparation may proceed; M5 jobs wait for the current main check and2191 finite paired measurements. Dispatch requires the ready candidate and coordinator release.
- 2026-10-18 next: Component review accepted b85880f25dc489c725ab85ad82eb9f6cba66c902. Dispatched the explicitly approved manual Windows-only ci.yml run39159496454 with windows_only=true; verified run headSHA equals candidate. Other CI jobs are skipped. Actual Windows outcome and targeted M5 static workflow checks remain required before final acceptance; M5 checks wait only for2191 finite measurement collection.

Pre-code review scope correction: update the existing workflow contract for the expanded Windows-only label, require the stdio command, and exercise failure propagation for all four existing step commands. The current module still expects the former label and covers only three commands. No new test framework or workflow inputs.
