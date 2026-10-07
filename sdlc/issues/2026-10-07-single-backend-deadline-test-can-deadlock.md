# The single-backend deadline test can deadlock on the gate host

Status: open

## Finding

During the 2026-10-07 gate queue, the Rust test
`entities::article::search::tests::deadline::single_backend_deadline_keeps_fetched_rows_and_names_the_source`
hung twice on the gate host (the 2020 checkout at 4d85dff69 and the
2024 checkout at c037bddb3, both carrying main's article code). The
second hang was captured before the kill:

- Test binary state S, `wchan: futex_doWait` — an internal futex wait,
  not a socket or IO wait. Six threads.
- Forensics saved on the host: `yellow-hang-forensics-172247.txt`
  (process list, wchan, fds, task list, listening sockets).
- The test binary inherits the yellow gate lock fd (fd 3 =
  `.yellow-gate.lock`), so a hung test holds the shared lock until the
  binary dies. The first hang blocked the queue for hours behind this.

The test passes in other runs the same day (the 2018-branch yellow run
passed it; reruns pass). The hang is intermittent — a race inside the
test's own threading (the in-test fixture server plus the deadline
future), not an environment conflict.

## Second hang (same day)

The 2024 checkout's test-phase rerun hung on
`deadline_expiry_without_rows_names_the_deadline_error` with the
identical signature (futex_do_wait, state S) at zero load and 13 GiB
free — not resource pressure. Forensics:
`yellow-hang-forensics-*.txt` (second file). The identical suite
passes on GitHub's runners the same day (branch CIs and two
merged-tree runs green). Host-specific intermittence until the stress
loop says otherwise.

## Gate-evidence rule while this stays open

A landing's yellow evidence is: lint and spec yellow-green, the test
phase attempted on yellow with any hang forensically captured and
recorded here, and the identical suite green on the branch CI and the
merged-tree CI. The issue must close or be accepted by Ian before the
0.9.2 tag.

## Triage questions

1. Reproduce under stress (loop the single test with nextest on a
   scratch checkout) and capture a thread dump at the futex
   (tokio-console or a gdb attach needs no root for a same-user
   process).
2. The test's watchdog scaling under `NEXTEST_EXECUTION_MODE` — check
   whether the deadline future and the fixture server share a runtime
   that can stall.
3. Consider a test-side timeout wrapper (the repo's own watchdog
   pattern) so the test fails itself instead of hanging the lane.

## Disposition

Blocker for gate reliability, not for correctness of any open ticket:
both affected runs were restarted and the queue proceeds. Fix on its
own ticket before the 0.9.2 tag if the stress loop reproduces quickly;
otherwise record as a known flake with the forensics path.
