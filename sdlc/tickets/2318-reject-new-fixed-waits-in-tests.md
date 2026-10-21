---
priority: 2
---

# 2318 — Reject new fixed waits in tests

Status: COMPLETE.

Milestone: 0.9.4

Depends on: 2301, 2302, 2303

Reviews: revision 5fd9b4a, accept

Reviews: revision 3cd3a05, accept

## Outcome

After 2301 and 2302, one routine check rejects a new fixed sleep or short fixed deadline in a test and any test that mixes a paused clock with a real socket. Existing sleeps that remain are listed once with their reason.

- Proof: The check fails on a planted test with a fixed sleep and passes on main.

## Evidence

35 Rust files call sleep and Python tests used 14 short fixed deadlines; fixed waits caused 2298, 2178 and 2175 (root cause record 2299). From Ian's process assessment of 2026-10-20.

Changes: `tools/check-quality-ratchet.py`, `tests/test_quality_ratchet_contract.py`, `tools/test-wait-exceptions.json`, `tests/managed_state_permissions.rs`, `CHANGELOG.md`.
