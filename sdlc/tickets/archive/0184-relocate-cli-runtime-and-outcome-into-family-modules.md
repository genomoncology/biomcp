---
flow: build
priority: 6
---
# Relocate CLI runtime and outcome into family modules

After the CLI command payloads move into per-entity family modules, the next slice is relocating the runtime behavior: helper functions, dispatch handler bodies, and the execution seam (`run()`, `execute()`, `run_outcome_inner()`).

Completed under March on 2026-04-13, as March ticket 184. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/184-relocate-cli-runtime-and-outcome-into-family-modules
