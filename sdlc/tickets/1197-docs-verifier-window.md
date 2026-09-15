---
flow: build
priority: 1
deps: []
---

# 1197: The docs-publication security probes stop racing the wall clock

## Goal

`tests/test_docs_publication_contract.py::test_live_verifier_cache_busts_and_fails_closed_on_wrong_paths`
passes under any host load: the unexpected-path and unexpected-origin probes
decide on a frozen clock, so a starved host can no longer expire their
0.01-second budget before the requests are admitted.

## Current Facts

Under 3× CPU oversubscription on the gate host (4 cores, 12 spinners), the
saturated full-system run at `1f917bfe` failed this test once with
`Expected regex: 'unexpected path'` and
`Actual message: 'publication did not converge after 0.01s: no response'`;
the test passes solo 3/3 in about 0.1 seconds.

Mechanism: with `timeout_seconds=0.01` and the default real
`time.monotonic`, `verify_publication` (`scripts/verify-docs-publication.py`)
sets `deadline = monotonic() + 0.01` before its inventory walk, and its
top-of-loop guard raises
`VerificationError("publication did not converge after <t>s: <last_error>")`
once the budget has elapsed — which starved scheduling achieved before the
first request was admitted, masking the security mapping that the regex
asserts.

The two probes prove the redirect security mappings (`SecurityViolation` for
a wrong final path and for a cross-origin final URL). The `0.01` values
arrived with the original test authoring (`ab063ef3`) and no other coverage
depends on a real-clock budget at those sites; every other deadline-sensitive
call in the file already injects a clock (`mismatch_clock` and the retry
test's advancing `monotonic` helpers). The verifier script is publication CI
infrastructure (`scripts/`, consumed by `.github/workflows/docs-edge.yml`),
not shipped runtime.

## Scope

Change only the two security probe calls in the failing test: inject a local
`frozen_clock` (a `monotonic` that returns 0.0) so the deadline window cannot
elapse before admission. No assertion is added, removed, weakened, or
reworded; no timeout constant changes; the verifier script and every other
file are untouched. Deadline behavior in this test remains covered
deterministically by the `mismatched_index` block (its injected advancing
clock drives expiry and asserts the wrapped "live bytes do not match"
message) and by the 601 validation case.

## Tests

The failing test itself gains the frozen-clock injection at its two security
probes. No new test functions.

## Acceptance

The single test passes 3/3 under twelve spinners on the four-core gate host —
the exact conditions of the observed failure — and the full
`tests/test_docs_publication_contract.py` passes there under the same
spinners. Ruff check and format are clean and the quality ratchet passes.

## Evidence

- Pre-fix, a starvation-simulated clock (0.0 then 999.0) reproduces the exact
  saturated-run message `publication did not converge after 0.01s: no
  response`; the frozen clock yields
  `SecurityViolation: <witness> resolved to unexpected path /wrong.txt`,
  which the regex asserts.
- Dev host (16 cores): single test passes under 12 spinners and under 48
  spinners (3× oversubscription, mirroring the failure).
- Gate host (4 cores): single test passes 3/3 under 12 spinners; the full
  file passes 8/8 under the same spinners.
- This change touches no timeout constant; the verifier script is unchanged.

## Dependencies

None.

## Complexity

- Contract score: 0 (one exact existing rule: keep the probes' security
  mapping observable)
- State and timing score: 0 (pure local test-harness clock; synchronous, no
  async or shared state)
- Reach score: 0 (one test function in one file)
- Proof score: 1 (deterministic both-directions demonstration plus
  saturated-host empirical runs at the failure conditions)
- Cost of error score: 0 (test-only, cheap local correction)
- Total: 1
- Minimum level floor: none
- Final level: 1
- Reasons: single-site clock injection with a deterministic proof pair; no
  concurrency, recovery, or shared-state floor applies
- Selected model: gpt-5.6-luna, high reasoning (level 1 implementer)

## Review

- Design review: not required — level 1 test-only fix with a frozen-mechanism ticket record; the reviewer confirmed rubric-correct level 1
- Code review: ACCEPT 2026-09-15 — diff verified as exactly the two frozen-clock injections plus this ticket; mechanism traced into scripts/verify-docs-publication.py line by line; coverage reduction ruled out; report-only notes on the constant-clock spin risk (unreachable today) and the imprecise 'every other call' sentence
