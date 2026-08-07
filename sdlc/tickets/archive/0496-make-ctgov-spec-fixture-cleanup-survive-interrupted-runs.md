---
flow: quickfix
priority: 5
---
# Make CTGov spec fixture cleanup survive interrupted runs

The 2026-07-10 audit found 59 orphaned `uv run` supervisors under PID 1, each with a Python CTGov fixture-server child, from completed BioMCP worktrees up to 22 days old. Roughly 118 stale processes survived March runs. A normal completed `make spec` cleaned its current fixture, so the failure mode is interruption or a runner path that bypasses per-block cleanup.

Completed under March on 2026-07-10, as March ticket 496. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/496-make-ctgov-spec-fixture-cleanup-survive-interrupted-runs
