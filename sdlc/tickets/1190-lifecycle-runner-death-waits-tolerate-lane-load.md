---
flow: build
priority: 1
deps: []
---

# 1190: Lifecycle runner-death waits tolerate lane load

## Goal

The runner-termination lifecycle tests wait long enough for a signaled
`run-specs.sh` to finish tearing down its fixtures under full test-lane load,
on both hosts, without weakening any cleanup assertion.

## Current Facts

After signaling the runner (or externally killing a fixture owner), the
lifecycle tests assert a ten-second bounded wait for process death: article
runner waits at tests/test_article_spec_fixture_lifecycle.py:283, 321, 774
(`128 + signal` exit codes), the ctgov runner wait at
tests/test_ctgov_spec_fixture_lifecycle.py:126, and the disease-survival
owner and bounded-runner SIGKILL waits at
tests/test_disease_survival_fixture_lifecycle.py:100, 166, 368
(`-signal.SIGKILL`). Teardown
kills fixture process groups, supervisors, and servers before exit. Under
four-way lane load these waits exceeded ten seconds on both hosts: four such
tests (thirteen parametrized cases) failed in the full `make test` lane on
the 16-core host at rebased main (2026-09-12, f9090392 run) and nine on the
4-core host, always as `subprocess.TimeoutExpired` on the wait. Sites 774,
100, and 166 are widened preventively: they share the exact post-kill
ten-second pattern but were not in the observed failure set. All thirteen
observed-failure cases pass solo on both
hosts (the gate-host solo pass of the bounded-runner case predates the
  deterministic uutils diagnosis: it passed solo there earlier the same day
  before coreutils behavior was implicated, and is deterministic only under
  the wrapper construct the amendment removes). The failures are the
documented class in
sdlc/issues/2026-09-12-spec-fixture-lifecycle-runner-tests-time-out-under-host-pressure.md
and are the only red in otherwise green gates.

## Scope

Raise the post-signal `wait` timeout from ten seconds to sixty seconds at the
named assertion sites in the three lifecycle test files. No production code,
no fixture scripts, no runner changes, no assertion changes: exit codes,
cleanup checks, and `_wait_until` deadlines stay exactly as they are.

Amendment 2026-09-12, after the sixty-second widening still left nine lane
failures on the gate host and a dedicated investigation captured the stuck
runners live. Two root causes, both now understood:

- Deterministic, gate-host-only: Ubuntu 25.10's uutils coreutils 0.2.2
  `timeout --signal=KILL` never delivers the kill — minimal proof:
  `timeout --signal=KILL 2s sleep 5` on the gate host ran the full five
  seconds and exited 124, while GNU on the dev host killed at two seconds
  with exit 137. `test_real_bounded_runner_timeout_reaps_disease_server_and_root`
  wraps its runner in exactly this construct
  (tests/test_disease_survival_fixture_lifecycle.py:350-361), so the wrapper
  never fires, the wait at :368 times out, and the runner bash is orphaned in
  the script's hold loop (run-specs.sh:634-640) — orphans were captured alive
  at three hours fifty-four minutes.
- Intermittent, both hosts: CPU-starvation sensitivity of the fork-heavy
  EXIT-trap teardown. All article and ctgov lifecycle tests pass solo on both
  hosts, pass the full trio on an idle gate host, and passed the complete
  pytest lane on an idle gate host; the failures occur only while other work
  saturates cores. Every captured runner sits in `do_wait` on a one-second
  sleep — no lock waits, no D-states. The sixty-second widening stays and the
  operational rule stands: the gate host runs nothing else during gates.

Amendment scope: in the disease test, drop the `timeout --signal=KILL 3s`
wrapper entirely and enforce the three-second bound in Python — a timer that
  SIGKILLs the Popen pid directly, with no coreutils group semantics on
  either flavor — keeping the `-signal.SIGKILL` exit assertion and every
  cleanup assertion unchanged. Two constraints are load-bearing: the
  three-second timer starts at Popen so it bounds runner start through full
  fixture standup exactly as the wrapper's clock did, and the kill is
  pid-only with the runner spawned `start_new_session=True` (as the sibling
  owner-death test does), because without the wrapper the runner bash would
  share pytest's process group and any group-directed kill would SIGKILL
  pytest itself.

## Acceptance

The three lifecycle files pass solo on both hosts, and a full `make test`
lane on both hosts completes with zero lifecycle failures. The other lane
results are unchanged.

## Dependencies

None.

## Complexity

- Contract score: 0 (one exact existing rule: same assertions, wider wait)
- State and timing score: 0 (pure test-harness wait budget)
- Reach score: 0 (three test files, one constant each site)
- Proof score: 1 (solo deterministic plus full-lane empirical on both hosts)
- Cost of error score: 0 (test-only; a too-generous wait cannot hide a wrong
  exit code or missing cleanup)
- Total: 1
- Minimum level floor: none
- Final level: 1
- Reasons: originally mechanical wait-budget widening; amendment adds the
  wrapper removal and in-test pid SIGKILL, copying the proven sibling pattern
  at tests/test_disease_survival_fixture_lifecycle.py:81-107
- Selected model: gpt-5.6-luna, high reasoning (level 1 implementer)

## Review

- Amendment 1 design review: ACCEPT 2026-09-12 with no blockers; four text
  corrections incorporated (citations 350-361 and :368, solo-pass
  reconciliation, complexity wording, and the two load-bearing constraints:
  timer at Popen, pid-only kill with start_new_session). Equivalence proven:
  supervisor and server were never in the wrapper's killed group; owner-death
  detection is pid-based; the exit assertion strengthens.
- Amendment 1 code review: ACCEPT 2026-09-12 at 5eefd94c; one file, 8
  insertions, 1 deletion, confirmed by the primary agent with git show --stat;
  constraints verified, loop termination proven, ordering race ruled out.
  Report-only: kill lands within one 0.05s poll interval of the deadline.
- Original design review: ACCEPT 2026-09-12 with no blockers; three ticket corrections
  incorporated (disease-site wording, thirteen-case count, preventive sites
  774/100/166, slug filename). Residual risks noted: article:817 setup wait
  and routine-fixture-recovery post-kill waits share the exposure class and
  are deliberate exclusions for now.
- Code review: ACCEPT 2026-09-12 at commit e9698a57; seven constants verified
  by full-file comparison, no assertion or _wait_until changes, runner script
  byte-identical, no formatting churn. Byte-level closure by the primary
  agent: 3 files, 7 insertions, 7 deletions.
