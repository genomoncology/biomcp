# 2148 — contract gates in the pm-job checkout (build record)

Gates green at code tip `46c466aa9`; branch tip `22f295ff9` adds only
this record and the ticket's job ids. Stopped without landing. The gate is
green in the pm-job checkout shape that was red at main.

## What ran (pm job, the Linux gate host, repo biomcp)

- `48b53161a` (main tip, run `1792205143115-2093501`): `make test-contracts`
  failed exactly the two tests the ticket names, 1157 passed, 3 skipped.
- `46c466aa9` (branch tip, run `1792205667515-2116977`): `make test-contracts`
  passed, 1160 passed, 3 skipped, in the same `worktrees/biomcp-2148-job`
  checkout.
- `46c466aa9` (branch tip, run `1792206125328-2146926`): `make lint` passed.

## Mechanism each test tripped on

The offline sandbox (`tools/run-offline`) binds a fresh host temp directory
at `/tmp` and sets `TMPDIR=/tmp` inside the namespace. The routine lane's
`env TMPDIR=...` pass-through on the pytest recipe undid that for
`make test` and its CI halves, but `make test-contracts` ran outside the
lane, so:

- `test_python_contract_temporary_paths_stay_in_worktree` saw
  `tmp_path = /tmp/pytest-of-ian/pytest-0/popen-gw1/...`, outside the
  checkout, so `ROOT in tmp_path.parents` failed. Purely the missing lane
  environment; the checkout's git worktree metadata and `.git` file pointer
  were never read.
- `test_health_distinguishes_clean_and_dirty_working_trees` built its
  fixture checkout on that same sandbox `/tmp` filesystem, whose free
  space read 7,683,672 KiB. `sdlc/project/health` reports a checkout unfit
  below its 10 GiB dispatch floor (`/tmp has 7683672 KiB available; floor
  is 10485760 KiB`), so both the clean and the dirty run exited 1 and
  `clean.returncode == dirty.returncode == 0` failed. The worktree/pointer
  hypotheses from the ticket are ruled out by that stdout line.

The macOS gate host was one ambient-TMPDIR accident away from the same red:
outside the sandbox the ambient `TMPDIR` sits outside the worktree too, and
only the routine lane's redirection kept the suite green there.

## What changed

- `Makefile`: the `MAKECMDGOALS` filter that exports the worktree `TMPDIR`
  and `BIOMCP_ROUTINE_TEST_LANE` now covers `test-contracts` and
  `test-contracts-prepared`, and `test-contracts` depends on
  `prepare-routine-test-tmp` first, so the directory exists before the
  lane's exported `TMPDIR` carries the build.
- `tests/test_upstream_planning_analysis_docs.py`: the pinned
  `test-contracts:` recipe line follows the new prerequisite.
- `tests/test_python_contract_parallelism.py`: a new dry-run contract
  asserts the gate carries `TMPDIR` and `--basetemp` into the worktree, so
  a silent drop of the lane fails the suite, not the next gate host.
- Ticket `2148` filed with the evidence and both mechanisms.

No changelog bullet: the diff touches no shipped path (`src/`, `templates/`,
`skills/`, `docs/`, `crates/`, or the pinned root files), so the release
coverage check needs none.

## Keeps

Both tests assert exactly what they asserted on main; nothing was skipped,
xfailed, or loosened. `tools/run-offline` keeps its sandbox contract,
including `TMPDIR=/tmp` inside the namespace. The CI lanes keep running
`make test-python`; with the lane now shared, `make test-contracts` and the
CI lane run the same command shape.

## Deferred

- No run on the macOS gate host: its routine gates were never red, and CI
  proves the shared lane shape on Linux. The coordinator can add one gate
  run there at landing if it wants the cross-host line.
- `make test` (the routine offline gate) on the Linux gate host at this tip
  is the closure gate the coordinator runs; this branch ran the contract
  gate and lint it changes.

## Cleanup

The ticket worktree stays for review. No scratch was left in `/tmp`.
