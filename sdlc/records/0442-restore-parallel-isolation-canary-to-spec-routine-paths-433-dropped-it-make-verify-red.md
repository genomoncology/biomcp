---
base: 956106eeb19041283f94fdef8dfdb22e7cd3a3bc
head: 886337132d23d36789ec0e1cee67302f6d0d64c1
---
Re-add test_parallel_isolation_contract.py to SPEC_ROUTINE_PATHS in\ \ run-specs.sh (433 commit 1b5143f2 dropped it); restores the cli.md Validation-Lanes-Stay-Split\ \ contract and the parallel-isolation guard. Last make verify red \u2014 unblocks\ \ the v0.8.24 tag.

Imported from March ticket 442. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/442-restore-parallel-isolation-canary-to-spec-routine-paths-433-dropped-it-make-verify-red
