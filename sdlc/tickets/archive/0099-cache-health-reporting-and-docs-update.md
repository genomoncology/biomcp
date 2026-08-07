---
flow: build
priority: 5
---
# Cache health reporting and docs update

After the runtime paths and migration helper land (T101, T102), shipped operator docs and example output still hardcode the old `http-cacache/` directory name and `/tmp/biomcp/` download location. This ticket refreshes all user-facing documentation and executable spec references to match the settled runtime contract.

Completed under March on 2026-04-01, as March ticket 099. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/099-cache-health-reporting-and-docs-update
