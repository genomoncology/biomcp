---
flow: quickfix
priority: 9
---
# Restore parallel-isolation canary to SPEC_ROUTINE_PATHS (433 dropped it; make verify red)

Re-add test_parallel_isolation_contract.py to SPEC_ROUTINE_PATHS in\ \ run-specs.sh (433 commit 1b5143f2 dropped it); restores the cli.md Validation-Lanes-Stay-Split\ \ contract and the parallel-isolation guard. Last make verify red \u2014 unblocks\ \ the v0.8.24 tag.

Completed under March on 2026-06-23, as March ticket 442. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/442-restore-parallel-isolation-canary-to-spec-routine-paths-433-dropped-it-make-verify-red
