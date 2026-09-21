---
flow: build
priority: 4
deps: []
---

# 1220: Restore the canonical CI gate on main

## Goal

`canonical-gates` passes on main so the lint, test, and spec guards actually run, including the container-publication spec and provenance tests. Today it fails before reaching any gate: the pinned `bubblewrap=0.9.0-1ubuntu0.1` is gone from the `ubuntu-24.04` runner image (noble-updates now carries `0.9.0-1ubuntu0.3`), and once that installs, `make test` fails on the stale `MAX_PACKAGE_FILES = 1_342` while the tree packages 1,347 members.

## Current Facts

- Runs 35653227218, 35657828871, and 35658018242 all fail `canonical-gates` at `Install canonical gate tools` with `E: Version '0.9.0-1ubuntu0.1' for 'bubblewrap' was not found` (ci.yml:21, :56-60).
- noble-updates and noble-security carry `bubblewrap 0.9.0-1ubuntu0.3`; `apparmor` and `apparmor-profiles` at the pinned `4.0.1really4.0.1-0ubuntu0.24.04.7` and `ripgrep 14.1.0-1` are still current, but all four are exact apt patch pins that the runner image can move.
- On yellow at 3111e783, `make lint` and `make spec` pass and `make test` leaves only `tests/test_source_package_boundary.py` failures: `MAX_PACKAGE_FILES = 1_342` (test_source_package_boundary.py:20) against 1,347 packaged members. The same two tests fail at 3b1d6452, before ticket 1219.
- The docker guards run in `make spec` (`spec/surface/docker-image.md` in spec-static) and `make test` (`tests/test_release_workflow_provenance.py`), both invoked by `canonical-gates` (ci.yml:86-90). While `canonical-gates` is red, those guards do not run.

## Design

- `ci.yml`: stop pinning exact apt patch versions for `bubblewrap`, `apparmor`, `apparmor-profiles`, and `ripgrep`; install the runner's current versions with `--no-install-recommends`. Keep the AppArmor boundary steps (`apparmor_parser -r`, `sysctl kernel.apparmor_restrict_unprivileged_userns`, `tools/run-offline -- true`) as the real check, and keep the non-apt tool pins. Delete the four now-unused version env vars.
- `tests/test_source_package_boundary.py`: set `MAX_PACKAGE_FILES` to the measured count and extend the comment with the tickets that moved it.
- Acceptance: `canonical-gates` is green on main after the push, and `make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA, so the docker spec and provenance tests run.

## Out of scope

- The rare `async_io_crossing_expiry_settles_without_admitting_a_mutation` Rust flake.
- Other CI jobs.

## Review

- Design review: not required, mechanical fix forced by the observed CI failure
- Code review: pending
