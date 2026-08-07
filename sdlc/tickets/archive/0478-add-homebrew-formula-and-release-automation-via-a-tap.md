---
flow: build
priority: 5
---
# Add Homebrew formula and release automation via a tap

A `brew install` path serves the large Mac-native developer audience that does not use `uv`/`pip`. BioMCP's single self-contained binary is ideal for a Homebrew formula. The ongoing cost is a per-release formula bump, which should be automated so it is not manual toil.

Completed under March on 2026-07-01, as March ticket 478. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/478-add-homebrew-formula-and-release-automation-via-a-tap
