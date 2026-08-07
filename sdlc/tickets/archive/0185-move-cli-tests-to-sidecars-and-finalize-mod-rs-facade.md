---
flow: build
priority: 6
---
# Move CLI tests to sidecars and finalize mod.rs facade

After CLI payloads and runtime dispatch move into family modules, the final slice is test relocation and bringing `src/cli/mod.rs` under the 700-line cap. 5,000+ lines of tests currently live inline in mod.rs; they need to move next to the code they exercise.

Completed under March on 2026-04-14, as March ticket 185. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/185-move-cli-tests-to-sidecars-and-finalize-mod-rs-facade
