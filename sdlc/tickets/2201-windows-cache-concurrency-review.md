---
priority: 2
---

# 2201 — Review Windows cache concurrency

Status: OPEN.

Milestone: 0.9.4

## Outcome

An agent test review on Windows covers parallel CLI runs, an MCP server and `cache clear` sharing one cache, and the size marker, under Windows' mandatory locks.

- Evidence: the 0.9.3 release review ran on Linux only; 2170's Windows CI case covers lock contention but not the full concurrent matrix.
- Changes: `tests/windows_stdio_contract.rs` `.github/workflows/ci.yml` `CHANGELOG.md` `sdlc/records/2201-windows-cache-concurrency-review.md`. Extend the existing portable threaded stdio fixture and manual Windows-only check; no automatic triggers or product patches.
- Proof: a recorded run on Windows with results; file defects as tickets.

## Progress

- 2026-10-18 started

PM decision: Ian approved a bounded manual GitHub Windows run for this concurrency matrix; no Windows host exists. Keep dispatch manual and scoped. Pre-code plan: reuse the existing recorded MyGene BRAF reply and Windows threaded StdioMcp; cover simultaneous CLI and persistent-MCP cache writes, warm reuse, cache clear while the MCP process remains alive, post-clear refill/reuse, the valid byte marker and unrelated sibling preservation. Use request-arrival barriers to prove overlap without inventing an ordering contract that can deadlock against legitimate operation exclusion. Preserve the approved conservative-overcount bound until maintenance. Report defects separately. Code preparation may proceed; M5 jobs wait for the current main check and2191 finite paired measurements. Dispatch requires the ready candidate and coordinator release.
