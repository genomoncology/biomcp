---
flow: build
priority: 4
deps: []
---

# 1220: Restore the canonical CI gate on main

## Goal

`canonical-gates` passes on main so the lint, test, and spec guards actually run, including the container-publication spec and provenance tests. Today it fails before reaching any gate: the pinned `bubblewrap=0.9.0-1ubuntu0.1` is gone from the `ubuntu-24.04` runner image (noble-updates now carries `0.9.0-1ubuntu0.3`).

## Current Facts

- Runs 35653227218, 35657828871, and 35658018242 all fail `canonical-gates` at `Install canonical gate tools` with `E: Version '0.9.0-1ubuntu0.1' for 'bubblewrap' was not found` (ci.yml:21, :56-60).
- noble-updates and noble-security carry `bubblewrap 0.9.0-1ubuntu0.3`; `apparmor` and `apparmor-profiles` at the pinned `4.0.1really4.0.1-0ubuntu0.24.04.7` and `ripgrep 14.1.0-1` are still current, but all four are exact apt patch pins that the runner image can move.
- `MAX_PACKAGE_FILES = 1_342` is already correct at d4b2758a: `cargo package --list` and the packaged `.crate` both carry 1,342 members when pytest runs with `TMPDIR` inside the worktree, as `make test` sets it, and both package-boundary tests pass. The earlier 1,347 reading at 3111e783 and 3b1d6452 was an artifact of untracked files in the gate clone, not the committed tree.
- `tests/test_offline_gate_contract.py` asserted the pinned `BUBBLEWRAP_VERSION`, `APPARMOR_VERSION`, and `RIPGREP_VERSION` strings in `ci.yml`. `make test` runs that file, so the unpin requires updating those assertions alongside the workflow.
- The docker guards run in `make spec` (`spec/surface/docker-image.md` in spec-static) and `make test` (`tests/test_release_workflow_provenance.py`), both invoked by `canonical-gates` (ci.yml:86-90). While `canonical-gates` is red, those guards do not run.

## Design

- `ci.yml`: stop pinning exact apt patch versions for `bubblewrap`, `apparmor`, `apparmor-profiles`, and `ripgrep`; install the runner's current versions with `--no-install-recommends`. Keep the AppArmor boundary steps (`apparmor_parser -r`, `sysctl kernel.apparmor_restrict_unprivileged_userns`, `tools/run-offline -- true`) as the real check, and keep the non-apt tool pins. Delete the three now-unused version env vars.
- `tests/test_source_package_boundary.py`: leave `MAX_PACKAGE_FILES = 1_342` and its comment unchanged; the measured packaged count already matches the constant.
- `tests/test_offline_gate_contract.py`: assert the unpinned install (`--no-install-recommends` and the four package names) and keep the AppArmor boundary assertions.
- Acceptance: `canonical-gates` is green on main after the push, and `make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA, so the docker spec and provenance tests run.

## Out of scope

- The rare `async_io_crossing_expiry_settles_without_admitting_a_mutation` Rust flake.
- Other CI jobs.

## Review

- Design review: not required, mechanical fix forced by the observed CI failure
- Code review: pending
