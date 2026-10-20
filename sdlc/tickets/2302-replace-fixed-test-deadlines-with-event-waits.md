---
priority: 2
---

# 2302 — Replace fixed test deadlines with event waits

Status: OPEN.

Milestone: 0.9.4

## Outcome

Changes: `tests/test_cache_sqlite.py`, `tests/test_mcp_http_surface.py`, `tests/test_mcp_stdio_guidance.py`, `tests/test_mcp_2026_protocol.py`, `tests/test_replay_server_lifetime.py`, `tests/test_routine_cargo_feature_contract.py`, `tests/test_no_cache_managed_state.py`, `CHANGELOG.md`, `sdlc/records/2302-event-waits.md`.

Subprocess and server tests wait on the real event (readiness line, exit, reaped child, forwarded signal) under one generous outer test timeout, as 2298 does, instead of short fixed deadlines. The 14 remaining short deadlines across 7 test files, including test_cache_sqlite.py, test_mcp_http_surface.py and test_mcp_stdio_guidance.py, change.

- Proof: Each changed test passes with a controlled pause injected before its wait, and still fails when the awaited event never happens.

## Evidence

2298, 2178 and 2175 failed full runs on fixed deadlines that a correct program missed when the host stalled. See the root cause record `sdlc/records/2299-root-cause-of-test-only-check-failures.md`.
