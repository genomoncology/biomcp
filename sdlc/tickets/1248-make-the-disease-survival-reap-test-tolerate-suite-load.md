# Make the disease-survival reap test observe the kernel, not the clock

Rewritten 2026-09-25 on the basis of ticket 1252 (signal-based waits);
the original text guessed at a cause without evidence and that guess
is withdrawn.

## Problem

`tests/test_disease_survival_fixture_lifecycle.py` (the
`test_disease_survival_setup_reaps_ppid_one_marker_orphan` case)
sampled a 200 ms heartbeat file to decide whether the stale fixture
process had been collected and whether the decoy was still alive. One
sample cannot tell a slow process from a wrongly killed one — and the
second case is the real bug the test exists to catch. The file failed
once in a full `make test` run on yellow (17ac88cc gate, ticket 1247's
cycle) and passed three consecutive focused runs, but no mechanism
was identified. Ticket 1252 landed the fix as part of its conversion
of the known-flaky set.

## Design

Liveness reads `/proc/<pid>/stat` directly (alive and not a zombie —
the kernel's view, no side-effect file and no clock). The stale
fixture is proven collected by waiting for its process to be gone,
and the decoy is proven alive with the same check, so a wrongly
killed decoy fails the test immediately instead of after a sampling
race. The waits use the shared `tests/support.py` helpers (`proc_alive`
and `wait_until` with the `BIOMCP_TEST_TIMEOUT_SCALE` watchdog), and
the heartbeat sampler is deleted. The stress lane (`make stress`,
ticket 1252) runs this test pinned to one CPU with forced worker
parallelism, so a regression is reproduced under contention rather
than hoped away.

## Acceptance

- The test contains no heartbeat sampling and no unscaled waits.
- `make stress` runs it green pinned to one CPU (ticket 1252's lane).

## Review

- Design review: folded into ticket 1252's accepted design
- Code review: folded into ticket 1252
- Verification: yellow gate with `make stress` under ticket 1252

## Review

- Closed by ticket 1252's conversion (this ticket was rewritten on the
  signal basis first); see
  `sdlc/records/1248-make-the-disease-survival-reap-test-tolerate-suite-load.md`.
