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

## Root cause

`TestHttpFixture` serves each accepted connection from a `tokio::spawn` task, and the held-reply arm of that task called `release.lock().expect("held reply lock").recv()` at `src/entities/article/test_support.rs:87-88`: a `std::sync::mpsc` receive held under a `std::sync::Mutex`. That receive is a futex wait, so every held reply parked one runtime worker thread until the test dropped the sender at the end of its body, and later held connections queued on the same mutex. The deadline tests run a four-worker runtime, so four held connections left no worker to drive the runtime: the tests' own `watchdog(60)` timers and the search deadlines could not fire, the tests never failed, and `canonical-gates` ran to its 45-minute cap. The `futex_doWait` forensics in the deadlock issue match this mechanism exactly. A sibling defect sat at `src/entities/article/graph/tests.rs:1118`: the delayed full-text reply slept with `std::thread::sleep` inside the fixture handler, which runs inside the connection task, so a single-threaded `#[tokio::test]` runtime froze for the whole delay with the same effect on its timers.

## Success criteria

- No blocking receive, lock wait, or thread sleep inside a spawned async task anywhere in the article fixtures. Held replies wait on `tokio::sync::mpsc` `recv().await` under a `tokio::sync::Mutex` (the `HeldReplyGate` helper), and the delayed full-text reply holds the connection while a timer task releases it, so the runtime's timers keep firing while a fixture reply is held.
- Every deadline test whose fixture can hold a reply for the test's whole life runs inside `hang_guard`, a named per-test watchdog with a 120-second base bound scaled by `BIOMCP_TEST_TIMEOUT_SCALE`, so a future hang fails that test with its own name in the panic message instead of stalling the lane. `failed_primaries_still_return_answered_rows` constructs a gate but its handler intercepts every route that could reach it, so it cannot hold.
- The deadline scope passes three consecutive runs on the build host through `yr`, and `make test`, `make lint`, and `make spec` each pass once through `yr`.
- The interim gate-evidence rule in the deadlock issue retires at landing, once main runs `canonical-gates` to completion; this branch cannot produce that evidence before it lands.
