---
flow: build
priority: 7
---
# Extract CLI family payloads into per-entity modules

The first pass at decomposing `src/cli/mod.rs` (12,780 lines) landed the shared scaffolding: `src/cli/types.rs` (shared types) and `src/cli/test_support.rs` (test helpers). The remaining 12,000+ lines are dominated by command payload structs, subcommand enums, runtime dispatch, and per-entity tests.

Completed under March on 2026-04-13, as March ticket 183. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/183-extract-cli-family-payloads-into-per-entity-modules
