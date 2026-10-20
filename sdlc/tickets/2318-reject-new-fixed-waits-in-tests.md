---
priority: 2
---

# 2318 — Reject new fixed waits in tests

Status: OPEN.

Milestone: 0.9.4

## Outcome

After 2301 and 2302, one routine check rejects a new fixed sleep or short fixed deadline in a test and any test that mixes a paused clock with a real socket. Existing sleeps that remain are listed once with their reason.

- Proof: The check fails on a planted test with a fixed sleep and passes on main.

## Evidence

35 Rust files call sleep and Python tests used 14 short fixed deadlines; fixed waits caused 2298, 2178 and 2175 (root cause record 2299). From Ian's process assessment of 2026-10-20.
