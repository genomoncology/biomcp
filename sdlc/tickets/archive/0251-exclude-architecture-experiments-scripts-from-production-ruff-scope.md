---
flow: quickfix
priority: 4
---
# Exclude architecture/experiments scripts from production ruff scope

Two consecutive spike tickets (243 and 244) failed at `verify+merge` on trivial ruff lint in `architecture/experiments/{slug}/scripts/` — one on E731 (lambda assignment), one on F401 (unused import). Both scripts passed the spike flow's in-flow baseline but tripped the full `make check` at merge time, which stopped the queue, auto-paused the team, and required manual recovery. Production lint rules should not apply to scratch experiment scripts; the fix belongs in the ruff config, not in every spike's harden step.

Completed under March on 2026-04-20, as March ticket 251. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/251-exclude-architecture-experiments-scripts-from-production-ruff-scope
