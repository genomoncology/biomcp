---
flow: build
priority: 5
deps: []
---

# 1225: Stop the wheel binary from overflowing the execute stack

## Goal

The published PyPI wheel binary completes `search trial`, `drug trials`, and `drug interactions` without a stack overflow, and CI catches a wheel-only stack regression before publish. The 0.9.0 wheel aborts on those paths while the cargo release binary survives, so Python users on the primary install channel crash.

## Current Facts

- GitHub issue #282 (2026-09-22) reports the trial search, `drug trials`, and `drug interactions` paths aborting on `uv tool install biomcp-cli` (0.9.0, Ubuntu 24.04 x86_64).
- Reproduced from the published wheel: `uvx --from biomcp-cli==0.9.0 biomcp search trial --condition diabetes --limit 1`, `... search trial --criteria "anti-PD-1 therapy" --limit 3`, `... drug interactions apixaban`, and `... drug trials imatinib` all exit 134 with `thread 'biomcp-cli-execute' has overflowed its stack`; `get trial NCT04280705` works.
- `src/cli/outcome.rs:599` sets `EXECUTE_STACK_BYTES = 8 * 1024 * 1024` and spawns the execute thread with that explicit `.stack_size()`, so `RUST_MIN_STACK` cannot raise it.
- Branches `biodata/0132` and `biodata/0133` already carry a 16 MiB stopgap; `sdlc/issues/2026-09-17-pypi-wheel-binary-stack-overflows-on-trial-search.md` and the 0.9.1 backlog P1 record the fix direction.
- Maturin builds the same release profile (`pyproject.toml:45-48`, `Cargo.toml:142-146`), so the overflow is frame-size-sensitive across build environments rather than a second code path.

## Design

- Raise `EXECUTE_STACK_BYTES` in `src/cli/outcome.rs` to 16 MiB, matching the downstream stopgap. If a wheel built on the gate host still overflows, raise it until the four commands pass and record the measured need.
- Add a CI job that builds the wheel (maturin on ubuntu-24.04), installs it into a clean venv, and runs the affected commands, failing on SIGABRT or a `stack overflow` message. Accept exit 0 or a clean source error; the smoke checks the stack, not live data.
- Record the wheel build and command results for the fix SHA so the next reader sees the margin that was needed.

## Acceptance

1. A wheel built from the pushed SHA on the gate host runs the four commands without a stack overflow.
2. `EXECUTE_STACK_BYTES` is 16 MiB or the measured value, with the reason recorded.
3. CI has a wheel-install smoke job that fails on a wheel reproducing the 0.9.0 overflow and passes on the fix, with the negative control shown.
4. `make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- Refactoring the recursion or the execute-thread model.
- The BioData integration branches.
- Republishing 0.9.0; the fix ships in 0.9.1.

## Complexity

- Contract score: 1 (one exact existing rule: the execute thread's explicit stack margin)
- State and timing score: 1 (thread stack state across the wheel and release builds)
- Reach score: 1 (the PyPI install channel)
- Proof score: 2 (a wheel build plus four command paths, not a unit test)
- Cost of error score: 2 (the primary install path ships broken to users)
- Total: 7
- Minimum level floor: none
- Final level: 3
- Reasons: build-specific runtime failure with external wheel proof and a published-artifact cost
- Selected model: gpt-5.6-sol, medium reasoning (level 3 implementer)

## Review

- Design review: pending
- Code review: pending
