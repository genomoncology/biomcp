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
   conversion). The child writes `entered\n` on the raw stdout fd
   (fd 1 through `std::fs::File`, not `println!` — the libtest harness
   captures `println!`) after acquiring the lease, then blocks reading
   stdin. The parent spawns the child with `.stdin(Stdio::piped())`
   and `.stdout(Stdio::piped())` (today stdout is nulled), scans
   lines until the exact `entered` line appears (harness status lines
   precede it) — that scan is the wait — asserts the child is still
   alive, runs its assertions, then drops the stdin handle to release
   the child. If the parent dies, the child reads end-of-input and
   exits by itself, so no orphan survives. A watchdog deadline still
   wraps the handshake (marked per rule 4) so a broken child fails the
   test instead of hanging the gate; the watchdog scales per rule 3.
2. Convert the 1247 settle wait to observe the condition: the settle
   helper already loops; restore the cut diagnostic line-neutrally —
   collect the leaked path names in the existing loop instead of bare
   booleans and print them in the failure. `src/sources/gencc.rs` sits
   exactly at the 1000-line un-inventoried threshold, so the rewrite
   must stay line-neutral; if it cannot, add an inventory entry under
   a 1252 authorization with the smallest delta that fits.
   `src/sources/gencc/tests.rs` is pinned at baseline 1138 and gets
   its baseline updated with the conversion. Keep the settle deadline
   as a marked watchdog. Convert the 1248 reap check: read
   `/proc/<pid>/stat` for alive and not zombie instead of sampling a
   heartbeat, and wait on the stale process being gone through the
   Python helper's `wait_until(lambda: not proc_alive(pid))` (Python
   has no wait handle for a non-child pid). Rewrite ticket 1248's
   text to this design before working it — its "suite processes race
   the scan" guess is withdrawn.
3. Shared wait helpers. Rust: extend `src/test_support.rs` (a
   cfg(test) module, not a directory) with a pipe-handshake child
   helper and a `watchdog(secs)` Duration builder. Python:
   `tests/support/` gains `wait_until(predicate, watchdog)` that
   polls with short intervals but always inside a scaled watchdog,
   and a `proc_alive(pid)` reading `/proc/<pid>/stat`. One factor,
   `BIOMCP_TEST_TIMEOUT_SCALE` (float, default 1.0), multiplies the
   watchdogs built through these helpers in both languages, read once
   per process. Raw `sleep` calls that remain do not route through
   the factor; they are pinned by the ratchet (rule 4) and ratchet
   down instead.
4. A ratchet lint, `tools/check-test-wait-ratchet.py`, wired into
   `make lint`: scan test sources (Rust `tests/`, `src/**/tests*`,
   `#[cfg(test)]` bodies including whole cfg(test) files such as
   `src/test_support.rs`; Python `tests/`) for new
   `Instant::now() + Duration`, `thread::sleep`, `tokio::time::sleep`,
   `time.sleep`, and test helpers named `*heartbeat*` containing a
   bare sleep (the mechanical form of the issue's "single heartbeat
   samples" rule; the tree's only existing sampler is deleted by this
   ticket). Lines carrying a `watchdog: <reason>` comment pass — the
   helpers' own poll sleeps carry one. Existing occurrences are
   pinned in an inventory (`tools/test-wait-inventory.json`) with
   counts per file that only ratchet down; a new occurrence outside
   the inventory fails. This follows the size-inventory pattern the
   repo already runs.
5. A stress lane: `make stress` runs the known-flaky set (the GenCC
   lease tests, the settle test, the disease-survival lifecycle tests)
   pinned to a two-CPU set — the test invocations run under
   `taskset -c 0,1` (not the cargo build) with explicit worker counts
   (`-j` for cargo, `-n` for pytest), because the runners
   auto-serialize on the detected CPU count otherwise — repeated N
   times (default 3, `BIOMCP_STRESS_REPEAT`), with
   `BIOMCP_TEST_TIMEOUT_SCALE` defaulted to 6 inside the lane so its
   watchdogs absorb the deliberate contention. One-CPU pinning was
   tried first and deadlocks the pipe handshake child
   deterministically — the mechanism is unidentified and filed as
   `sdlc/issues/2026-09-25-single-cpu-affinity-deadlocks-the-handshake-child.md`;
   the lane guards against ever pinning to exactly one CPU. The
   yellow gate runs it after `make test` for tickets that touch those
   areas — starting with this one. A fix is proven by reproduction
   under stress, not three lucky green runs: the lane's red side is
   proven by the ratchet failing on the parent commit's exact files
   and by the never-signaling-child unit test; a live flake
   reproduction was attempted against the parent tree and did not
   complete, and no reproduction is claimed.

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
- The red side is proven mechanically (recorded here): the ratchet
  fails on the parent commit's exact polling shapes — a scratch root
  holding `git show 03d9ced6` copies of the two converted files
  reports `new unmarked timed waits in src/entities/gene/gencc/tests.rs
  (2)` (the 120 s deadline and the 5 ms poll this ticket deleted) and
  14 in `src/sources/gencc/tests.rs` (the child's release-file poll
  among the file's other pinned waits) — and the handshake itself
  fails fast on a child that never signals
  (`signaled_child_spawn_fails_when_the_child_never_signals`). A
  live reproduction of the parent's flake under load was attempted by
  building the parent tree in a scratch worktree and did not
  complete within the session; no reproduction was observed or
  claimed. The stress lane runs green at the head SHA in the yellow
  gate.
- Yellow gate green at the head SHA, including `make stress`
  (two-CPU pinning; the one-CPU deadlock is a filed issue, not a
  gate failure).

## Review

- Design review: REJECT once (four P1s: the settle diagnostic could
  not land under the size gate, the heartbeat lint rule had no
  matcher, the scale factor's reach was overstated, and the stress
  acceptance had no red side), all folded, re-review ACCEPT 2026-09-25
- Code review: REJECT once (the ratchet tripped on its own test
  literals; two stale entries), fixed and verified 2026-09-25
- Verification: yellow gate at e01ebf7a lint/test/spec/stress OK; see
  `sdlc/records/1252-replace-timed-waits-with-signals-and-add-a-stress-lane.md`
