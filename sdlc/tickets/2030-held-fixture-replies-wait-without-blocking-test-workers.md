# 2030 — Held fixture replies wait without blocking test workers

Status: OPEN.

Milestone: 0.9.2

## Outcome

The article deadline tests never hang. A held fixture reply waits without blocking a runtime worker thread, and a hung test stops at a set time with its name instead of running until the CI job's 45-minute cap.

## Evidence

Filed 2026-10-08 from the third review of the work since v0.9.1 (ticket 2033, finding 1).

- `canonical-gates` hit its 45-minute cap in all seven main runs from `c51a19743` to `5248f291f` (37726916710 through 37757120357). The 2024 check run 37717723539 went green only on its seventh attempt.
- `src/entities/article/test_support.rs:87-88`: a held fixture reply calls a blocking `release.lock().recv()` inside `tokio::spawn`, and the tests run four worker threads. Each held request blocks one worker, and later held requests queue on the mutex (`futex_wait`, which matches the forensics in `sdlc/issues/2026-10-07-single-backend-deadline-test-can-deadlock.md`). With the workers blocked, the runtime's timers stop, including the test's own 60-second watchdog.
- The interim gate-evidence rule in that issue lets a landing pass with the Rust test phase only attempted. It treats a cancelled `canonical-gates` as a known flake, so those landings also skipped the spec gate on CI. Ian's approval of the rule is not recorded.

- Starts from: the deadlock issue file and ticket 2023's deadline tests.
- Keeps: the deadline behavior the tests pin, and the held-reply fixture's ordering guarantees.
- Changes: wait on an async primitive (`tokio::sync::Notify` or a oneshot channel) instead of a blocking receive; give the Rust test phase a per-test timeout that names the hung test; retire the interim gate-evidence rule.
- Proof: the deadline tests pass repeatedly under four workers on a loaded machine; main runs `canonical-gates` to completion; a planted hang fails at the per-test timeout with the test's name.
- Defers: nothing.
