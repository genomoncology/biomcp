---
flow: build
priority: 8
---
# Decompose health.rs into src/cli/health/ submodules

`src/cli/health.rs` is 3,181 lines and currently interleaves the health source catalog, HTTP probe transport, local-data and cache checks, concurrency/timeout orchestration, and a 1,600+ line inline test block. `biomcp health` is also an operator-facing readiness surface with explicit local-source coverage contracts, so the file needs a clear architecture without changing any visible behavior.

Completed under March on 2026-04-26, as March ticket 320. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/320-decompose-health-rs-into-src-cli-health-submodules
