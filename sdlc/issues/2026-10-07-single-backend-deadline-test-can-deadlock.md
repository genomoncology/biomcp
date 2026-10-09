# The single-backend deadline test can deadlock on the gate host

Status: closed by ticket 2030's landing (async holds, named hang guard)

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

## Interim rule (RETIRED 2026-10-08 — ticket 2030 landed)

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

## GitHub runners hit the same hang (2026-10-07 night)

The Finding's claim that the identical suite passes on GitHub's
runners no longer holds. Between 21:23Z and 00:24Z the CI workflow
cancelled canonical-gates at its 45-minute cap five times on this
signature, on main and on the 2024 fix branch, while one main run
(37695115787, 22:16Z) passed the whole lane in 23m54s:

- main 37689090490, 37691906207, 37695148578, 37696857007 — each
  cancelled at the cap in the Canonical test gate.
- branch tickets/2024-runner-windows-fix 37698460019, head d07642eb3
  (a branch that changes no Rust), attempt 2 — nextest reported SLOW
  above 2220s on `entities::article::search::tests::deadline::
deadline_expiry_without_rows_names_the_deadline_error` and
  `overall_deadline_returns_partial_rows_and_names_the_held_source`
  until the cap killed the job; the log shows two `biomcp_cli-*` test
  processes terminated as orphans. Attempt 1 cancelled the same step.
  Attempt 3 cleared the deadline tests and failed only on the source
  package file-count ratchet (the branch adds one test file; recounted
  in 059c61cb5).
- branch head run 37709695813 (059c61cb5, still no Rust changes) —
  SLOW above 2220s on `single_backend_deadline_keeps_fetched_rows_
  and_names_the_source`, the first test this issue names, until the
  cap. Third hang on the branch, three different tests of the same
  deadline module.

Main went green twice in ~20 minutes at 00:59Z and 01:00Z (runs
37710581502 and 37710742254) between these branch hangs, so the
intermittence is per run, not per branch.

Every other job in 37698460019, including windows-contracts, was
green. The interim gate-evidence rule's branch-CI clause cannot be
satisfied while a rerun can land on this deadlock; the coordinator
needs to treat a cancelled canonical-gates with this signature as
the known flake, or the cap has to rise until the fix lands.

## Closure (2026-10-08)

Ticket 2030 replaced the blocking receives with async primitives and
added hang guards naming the test; its branch CI and merged-tree run
are green, and it landed as merge ae9e6c2e9 (9b23236f8 is the follow-up records commit). The interim gate-evidence rule
above is retired: landings again require the ordinary green gates. The
pre-existing blocking waits outside the article fixtures (noted in the
2030 review) stay recorded here for a future ticket.
