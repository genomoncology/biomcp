---
flow: build
priority: 1
deps: []
---

# 2148: Make the Linux gate host's contract gates match the pm-job checkout shape

Status: OPEN.

Milestone: 0.9.3

Reviews: revision b9036da, accept

Reviews: revision 780dd2a, accept

## Outcome

`make test-contracts` is green in one pm job on the Linux gate host at the ticket branch tip, in the same pm-job checkout shape that is red today, without weakening either failing test.

## Evidence

Tonight, from the landing checks for 2137: `make test-contracts` in the Linux gate host's pm-job checkout (`worktrees/biomcp-NNNN-job`) fails exactly two tests that pass in CI and on the macOS gate host.

- `tests/test_source_package_boundary.py::test_python_contract_temporary_paths_stay_in_worktree` fails `assert PosixPath(<job worktree>) in <PosixPath.parents>`: the temp-path contract expects test temp paths inside the repo worktree, but under the gate's shape they are not.
- `tests/test_sdlc_project_lifecycle_contract.py::test_health_distinguishes_clean_and_dirty_working_trees` fails `assert 1 == 0`: the health script reports the test's fixture checkout unfit under the gate's shape.

Both fail identically at main's own SHA on the same host (job `1792198978016-2060578` at the main baseline, job `1792199646491-2074051` at the 2137 tip), so the cause is the job-checkout gate shape, not any landing. Reproduced at the current main tip in job `1792205143115-2093501`, which also names each mechanism.

Mechanism, confirmed on main: `make test` and its CI halves (`test-rust`, `test-python`) export `TMPDIR` into the worktree and pass `--basetemp` (the routine test lane, Makefile `MAKECMDGOALS` filter). `make test-contracts` ran outside that lane, so under the offline sandbox on Linux `TMPDIR` fell back to the sandbox's own `/tmp`, outside the worktree: the temp-path test found `tmp_path` at `/tmp/pytest-of-ian/...`, and the health script's fixture checkout then sat on that same `/tmp` filesystem, whose free space (7,683,672 KiB) sits below the script's 10 GiB dispatch floor, so health reported the clean fixture unfit (`/tmp has 7683672 KiB available; floor is 10485760 KiB`) and the clean/dirty returncode assertion failed. On the macOS gate host the ambient `TMPDIR` sits outside the worktree the same way, so the gate was one ambient-TMPDIR accident away from the same red. The checkout's git worktree metadata and `.git` file pointer are not involved: neither failing assertion reads them.

- Starts from: the `MAKECMDGOALS` filter above `export TMPDIR = $(ROUTINE_TEST_TMPDIR)` in `Makefile`, the `test-contracts:` target it omits, and the two tests named above on current main.
- Keeps: Both tests assert exactly what they assert on main. The CI lanes run `make test-python` and stay unchanged. `tools/run-offline` keeps its sandbox contract, including `TMPDIR=/tmp` inside the namespace.
- Changes: `Makefile` (the routine lane covers `test-contracts`, and its tmp dir exists before the build), `tests/test_upstream_planning_analysis_docs.py` (the pinned `test-contracts:` recipe line), `tests/test_python_contract_parallelism.py` (a dry-run contract that the gate carries the lane's `TMPDIR` and `--basetemp`).
- Proof: Red: pm job `1792205143115-2093501` on the Linux gate host at main's SHA `48b53161a` runs `make test-contracts` and fails exactly those two tests, with each failure's output naming its mechanism. Green: pm job `1792205667515-2116977` runs the same command in the same checkout shape at the branch tip and passes (1160 passed, 3 skipped); pm job `1792206125328-2146926` runs `make lint` at the same tip and passes. The landing's ci.yml run stays green.
- Defers: Nothing. The closure `make test` run on the Linux gate host belongs to the coordinator, and no gate run on the macOS gate host is needed: its routine gates were never red and CI proves the shared lane shape on Linux.
