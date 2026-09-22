---
flow: build
priority: 5
deps: []
---

# 1225: Ship the wheel in the release profile so it stops overflowing the stack

## Goal

The published PyPI wheel is built in the release profile, runs `search trial`, `drug trials`, and `drug interactions` without a stack overflow, and a pre-publish smoke catches a dev-profile wheel before it reaches PyPI.

## Current Facts

- GitHub issue #282 (2026-09-22) reports the trial search, `drug trials`, and `drug interactions` paths aborting on `uv tool install biomcp-cli` 0.9.0 (Ubuntu 24.04 x86_64).
- Reproduced from the published wheel: `uvx --from biomcp-cli==0.9.0 biomcp search trial --condition diabetes --limit 1`, `... search trial --criteria "anti-PD-1 therapy" --limit 3`, `... drug interactions apixaban`, and `... drug trials imatinib` all exit 134 with `thread 'biomcp-cli-execute' has overflowed its stack`; `get trial NCT04280705` works.
- The wheel is built in the dev profile. `.github/workflows/release.yml:151-156` calls `PyO3/maturin-action@v1` with only `toolchain` and `target`; the action's `args` input has no default, so `maturin build` runs without `--release`. The published `biomcp_cli-0.9.0-py3-none-manylinux_2_39_x86_64.whl` binary is 97,338,024 bytes with a 68,281,008-byte `.text`, while the release tarball binary is 32,526,272 bytes with a 24,055,888-byte `.text`. Unoptimized frames overflow the fixed stack on the deep paths; the release-profile tarball binary works.
- `src/cli/outcome.rs:599` sets `EXECUTE_STACK_BYTES = 8 * 1024 * 1024` with an explicit `.stack_size()`, so `RUST_MIN_STACK` cannot raise it.
- Ticket 1191 measured the debug-build margin and fixed the structural seam with `Box::pin` at `src/cli/outcome.rs:647`; it explicitly rejected raising the stack as papering over the margin. A downstream integration branch carries a 16 MiB stopgap (`src/cli/outcome.rs:637`), which has not landed on main and should not become main's fix.
- `sdlc/issues/2026-09-17-pypi-wheel-binary-stack-overflows-on-trial-search.md` and the 0.9.1 backlog P1 record the wheel smoke direction.

## Design

- Pass `args: --release --locked` to `PyO3/maturin-action@v1` in the `pypi-build` matrix so every wheel matches the release profile the tarballs use (`Cargo.toml:141-145`). `--locked` matches `ci.yml`'s lockfile discipline, so a drifted lockfile fails the release build by design. Verify by rebuilding the x86_64 wheel on the gate host and comparing its size and `.text` size against the tarball binary, not by reading the action input alone.
- Keep `EXECUTE_STACK_BYTES` at 8 MiB, per ticket 1191. If a release-profile wheel still overflows on the four commands, capture a gdb stack watermark on the gate host and stop and report with the measurement before considering any constant change; any such change is capped at 16 MiB with the same stop rule.
- Add a pre-publish smoke to `release.yml` between `pypi-build` and `pypi-publish`: install the built `wheel-x86_64-unknown-linux-gnu` artifact into a clean venv outside the repo tree (RUN.md warns that in-tree `uv run` rebuilds instead of proving the wheel) and run the four exact commands from Current Facts, failing on SIGABRT or a `stack overflow` message. Make `pypi-publish` need it. The smoke carries the `container_only` gate so a container-only dispatch still skips PyPI work.
- Update `docs/reference/release-process.md`'s job list and `tests/test_release_workflow_provenance.py` for the smoke job and its gating.
- Record the pre/post wheel size and `.text` size for the fix SHA as the profile evidence.

## Acceptance

1. A release-profile wheel built from the pushed SHA on the gate host has a `.text` section within a small margin of the 24,055,888-byte tarball binary (not the 68,281,008-byte dev build) and runs the four commands without a stack overflow.
2. `EXECUTE_STACK_BYTES` stays 8 MiB; any change requires the measured watermark, is capped at 16 MiB, and stops and reports if the cap does not clear the commands.
3. The release workflow's smoke fails on a dev-profile wheel and passes on the release-profile wheel, with the negative control recorded as a one-time gate-host demonstration (not a second dev-profile build on every release); `pypi-publish` depends on the smoke.
4. `make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- The missing Linux aarch64 wheel on PyPI.
- Refactoring the recursion or the execute-thread model.
- The downstream integration branches and their 16 MiB stopgap.
- Republishing 0.9.0; the fix ships in 0.9.1.

## Complexity

- Contract score: 1 (several explicit cases: the four command paths plus the two-binary `.text` size comparison)
- State and timing score: 0 (build configuration, no runtime state change)
- Reach score: 1 (the PyPI install channel)
- Proof score: 2 (wheel rebuild plus four command paths and a negative control, not a unit test)
- Cost of error score: 2 (the primary install path ships broken wheels to users)
- Total: 6
- Minimum level floor: none
- Final level: 3
- Reasons: build-profile defect in the published artifact with external wheel proof; the 0.9.1 backlog scored the outcome Level 2, but the rubric's proof and cost rows put it at 6
- Selected model: gpt-5.6-sol, medium reasoning (level 3 implementer)

## Review

- Design review: pending
- Code review: pending
