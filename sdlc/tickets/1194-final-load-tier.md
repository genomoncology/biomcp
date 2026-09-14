---
flow: build
priority: 1
deps: []
---

# 1194: Final load-sensitive test and spec tier

## Goal

Make the last three checks deterministic under the 3x CPU-oversubscribed
verification lane. Preserve every cleanup and projection assertion.

## Current Facts

The merged `fc81e388` lane used twelve busy loops on a four-core Yellow host.
Rust passed all 3,566 tests. One Python lifecycle test and two executable spec
blocks failed. The lifecycle failure occurred before the runner's ready marker
appeared. The runner's three-second Python kill timer had already killed the
runner, although its disease ownership record and fixture could still be
observed. The two spec failures produced no block output because the page-level
Mustmatch timeout killed the Bash wrapper before the Python block could print
its success marker.

The failures were:

- `test_real_bounded_runner_timeout_reaps_disease_server_and_root`, whose
  setup-record observation used the shared ten-second polling budget.
- `spec/entity/drug.md`, `Card command discovery`, a long provider-contract
  Python block.
- `spec/entity/gene.md`, `GenCC adapter projection parity`, a long adapter
  projection Python block.

## Scope

In the disease lifecycle test, wait for the ownership record directly with a
sixty-second observation budget. The runner ready marker is not required after
the intentional kill. Widen the two post-kill health and root observations to
sixty seconds. The direct pid-only SIGKILL and `-signal.SIGKILL` assertion stay
unchanged. These waits assert eventual fixture cleanup, not latency.

Give each long executable spec block a `timeout=600` fence directive. This
raises only the Mustmatch per-block budget. It does not alter the CLI, provider
fixture, expected projections, subprocess assertions, or production behavior.
The larger budget also preserves Mustmatch's explicit timeout diagnostic if an
oversubscribed block genuinely exceeds the bound.

## Acceptance

The disease lifecycle test passes with twelve busy loops running. The drug and
gene pages pass through the routine spec runner with twelve busy loops running.
The full parent gate verifies the merged lane.

## Complexity

- Contract score: 0 (existing fixture cleanup, projection, and timeout
  assertions remain exact)
- State and timing score: 1 (three observation budgets absorb scheduler
  starvation without changing observed outcomes)
- Reach score: 1 (one test and two executable spec pages)
- Proof score: 1 (targeted saturated lifecycle and page runs)
- Cost of error score: 0 (test and spec infrastructure only)
- Total: 3
- Minimum level floor: none
- Final level: 2
- Reasons: three test/spec-infrastructure budget fixes with saturated local proofs
- Selected model: gpt-5.6-luna, high reasoning (level 2 implementer)

## Review

- Design review: pending.
- Code review: pending.

## Proof

- With twelve busy loops: `test_real_bounded_runner_timeout_reaps_disease_server_and_root` passed in 3.21s.
- With twelve busy loops through a single-page copy of `scripts/run-specs.sh`,
  `spec/entity/drug.md` passed 15 Bash blocks.
- With twelve busy loops through a single-page copy of `scripts/run-specs.sh`,
  `spec/entity/gene.md` passed 22 Bash blocks.
