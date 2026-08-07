---
base: 37568a9a9a9aed8c5d0d9fa1a4ddb9aee87c4f71
head: 823574a3f7e10d26618d6f0691239de98b741e88
---
The 2026-07-10 audit found 59 orphaned `uv run` supervisors under PID 1, each with a Python CTGov fixture-server child, from completed BioMCP worktrees up to 22 days old. Roughly 118 stale processes survived March runs. A normal completed `make spec` cleaned its current fixture, so the failure mode is interruption or a runner path that bypasses per-block cleanup.

Imported from March ticket 496. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/496-make-ctgov-spec-fixture-cleanup-survive-interrupted-runs
