---
flow: build
priority: 1
deps: []
---

# 2148: Make the Linux gate host's contract gates match the pm-job checkout shape

Status: OPEN.

Milestone: 0.9.3

## Outcome

`make test-contracts` is green in one pm job on the Linux gate host at the ticket branch tip, in the same pm-job checkout shape that is red today, without weakening either failing test.

## Evidence

Tonight, from the landing checks for 2137: `make test-contracts` in the Linux gate host's pm-job checkout (`worktrees/biomcp-NNNN-job`) fails exactly two tests that pass in CI and on the macOS gate host.

- `tests/test_source_package_boundary.py::test_python_contract_temporary_paths_stay_in_worktree` fails `assert PosixPath(<job worktree>) in <PosixPath.parents>`: the temp-path contract expects test temp paths inside the repo worktree, but under the gate's shape they are not.
- `tests/test_sdlc_project_lifecycle_contract.py::test_health_distinguishes_clean_and_dirty_working_trees` fails `assert 1 == 0`: the health script reports the test's fixture checkout unfit under the gate's shape.

Both fail identically at main's own SHA on the same host (job `1792198978016-2060578` at the main baseline, job `1792199646491-2074051` at the 2137 tip), so the cause is the job-checkout gate shape, not any landing.

Candidate mechanism, to be confirmed on main before the fix: `make test` and its CI halves (`test-rust`, `test-python`) export `TMPDIR` into the worktree and pass `--basetemp` (the routine test lane, Makefile `MAKECMDGOALS` filter). `make test-contracts` ran outside that lane, so under the offline sandbox on Linux, `TMPDIR` falls back to the sandbox's own `/tmp`, outside the worktree; on the macOS gate host the ambient `TMPDIR` sits outside the worktree the same way. The checkout's git worktree metadata and `.git` file pointer are not involved: the failing assertions never read them.

- Keeps: Both tests assert exactly what they assert on main. The CI lanes run `make test-python` and stay unchanged. `tools/run-offline` keeps its sandbox contract, including `TMPDIR=/tmp` inside the namespace.
- Changes: `Makefile` (the routine lane covers `test-contracts`; its tmp dir exists before the build), `tests/test_upstream_planning_analysis_docs.py` (the pinned `test-contracts:` recipe line), `tests/test_python_contract_parallelism.py` (a dry-run contract that the gate carries the lane's `TMPDIR` and `--basetemp`).
- Proof: Red: one pm job on the Linux gate host at main's SHA runs `make test-contracts` and fails exactly those two tests. Green: the same command at the branch tip passes in the same checkout shape.
- Defers: Nothing.

## Work

1. Reproduce both failures in one pm job on the Linux gate host at main's SHA; confirm the mechanism each test trips on.
2. Bring `make test-contracts` into the routine test lane so the gate itself keeps temp paths in the worktree; update the Makefile shape contracts that pin the recipe.
3. Rerun `make test-contracts` in the pm-job checkout at the branch tip.

## Acceptance

1. One pm job on the Linux gate host at the branch tip runs `make test-contracts` green in the job-checkout shape.
2. Both tests keep their assertions from main; nothing is skipped, xfailed, or loosened.
3. ci.yml stays green on the landing tip.
