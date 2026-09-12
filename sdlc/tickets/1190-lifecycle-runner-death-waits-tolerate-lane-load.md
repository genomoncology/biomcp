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
hosts. The failures are the documented class in
sdlc/issues/2026-09-12-spec-fixture-lifecycle-runner-tests-time-out-under-host-pressure.md
and are the only red in otherwise green gates.

## Scope

Raise the post-signal `wait` timeout from ten seconds to sixty seconds at the
named assertion sites in the three lifecycle test files. No production code,
no fixture scripts, no runner changes, no assertion changes: exit codes,
cleanup checks, and `_wait_until` deadlines stay exactly as they are.

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
- Reasons: mechanical wait-budget widening with unchanged assertions
- Selected model: gpt-5.6-luna, high reasoning (level 1 implementer)

## Review

- Design review: pending
- Code review: pending
