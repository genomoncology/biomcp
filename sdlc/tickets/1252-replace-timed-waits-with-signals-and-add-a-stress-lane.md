# Replace timed waits in tests with signals and add a stress lane

From sdlc/issues/2026-09-24-tests-wait-on-the-wall-clock.md. Three
gate flakes (GenCC lease, GenCC settle, disease-survival reap) share
one cause: tests wait for events with fixed deadlines, polling loops,
and short samples, on a parallel 4-core host. Ticket 1239 lengthened a
deadline from 5 s to 120 s instead of removing the wait; ticket 1247
kept a 60 s deadline and dropped the promised diagnostic; ticket 1248
guessed at a cause from one 200 ms heartbeat sample.

## Problem

1. `src/sources/gencc/tests.rs:849` and
   `src/entities/gene/gencc/tests.rs:915` poll for up to 120 s. A
   slower host still exceeds them, and a parent panic leaves the child
   alive for up to 120 s.
2. `src/sources/gencc.rs:863` fails only after 60 s with a generic
   message, because the specific leaked-temp diagnostic was cut to
   stay under the file-size limit.
3. `tests/test_disease_survival_fixture_lifecycle.py:45-48,193`
   samples one 200 ms heartbeat; it cannot tell a slow decoy from a
   decoy the fixture wrongly killed (the second is a real bug).
4. Roughly 11 fixed-deadline lines in 7 Rust test files and 77 sleep
   or timeout calls in Python tests, with no ratchet against more.

## Design

1. Child handshake for the GenCC lease helper (the flagship
   conversion). The child writes `entered\n` to its stdout after
   acquiring the lease, then blocks reading stdin. The parent reads
   the line — that is the wait — asserts the child is still alive,
   runs its assertions, then closes stdin to release the child. If the
   parent dies, the child reads end-of-input and exits by itself, so
   no orphan survives. A watchdog deadline still wraps the handshake
   (marked per rule 4) so a broken child fails the test instead of
   hanging the gate; the watchdog scales per rule 3.
2. Convert the 1247 settle wait to observe the condition: the settle
   helper already loops; make it fail with the specific leaked path
   names when the deadline trips, restoring the cut diagnostic, and
   keep the deadline as a marked watchdog. Convert the 1248 reap
   check: read `/proc/<pid>/stat` for alive and not zombie instead of
   sampling a heartbeat, and wait on the stale process being gone via
   its handle (rewrite ticket 1248's text to this design before
   working it — its "suite processes race the scan" guess is
   withdrawn).
3. Shared wait helpers. Rust: extend `src/test_support/` with a
   pipe-handshake child helper and a `watchdog(scale, secs)` Duration
   builder. Python: `tests/support/` (or the existing helper module)
   gains `wait_until(predicate, watchdog)` that polls with short
   intervals but always inside a scaled watchdog, and a
   `proc_alive(pid)` reading `/proc/<pid>/stat`. One factor,
   `BIOMCP_TEST_TIMEOUT_SCALE` (float, default 1.0), multiplies every
   watchdog in both languages, read once per process.
4. A ratchet lint, `tools/check-test-wait-ratchet.py`, wired into
   `make lint`: scan test sources (Rust `tests/`, `src/**/tests*`,
   `#[cfg(test)]` bodies; Python `tests/`) for new
   `Instant::now() + Duration`, `thread::sleep`, `tokio::time::sleep`,
   `time.sleep`, and single-sample heartbeats. Lines carrying a
   `watchdog: <reason>` comment pass. Existing occurrences are pinned
   in an inventory (`tools/test-wait-inventory.json`) with counts per
   file that only ratchet down; a new occurrence outside the inventory
   fails. This follows the size-inventory pattern the repo already
   runs.
5. A stress lane: `make stress` runs the known-flaky set (the GenCC
   lease tests, the settle test, the disease-survival lifecycle tests)
   pinned to one CPU (`taskset -c 0`) with the suite's usual
   parallelism inside it, repeated N times (default 3,
   `BIOMCP_STRESS_REPEAT`). The yellow gate runs it after `make test`
   for tickets that touch those areas — starting with this one. A fix
   is proven by reproduction under stress, not three lucky green runs.

## Acceptance

- The GenCC lease tests contain no fixed sleep: the handshake is the
  wait, and the stress lane runs them pinned to one CPU green.
- Killing the parent mid-test leaves no orphan child (a test proves
  the child exits on end-of-input).
- The settle failure names the leaked paths.
- The ratchet fails on a new unmarked `time.sleep` in a test file and
  passes on the tree as landed.
- `BIOMCP_TEST_TIMEOUT_SCALE=10` visibly stretches a marked watchdog
  (unit test on the builder).
- Yellow gate green at the head SHA, including `make stress`.

## Review

- Design review: pending
- Code review: pending
