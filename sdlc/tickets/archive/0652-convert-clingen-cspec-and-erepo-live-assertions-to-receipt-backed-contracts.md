---
flow: build
priority: 10
---
# Convert ClinGen CSpec and ERepo live assertions to receipt-backed contracts

What is IN scope: - `src/entities/gene/cspec.rs`, `src/entities/variant/erepo.rs`, their source tests/captures, and the two live pages. - The corresponding entries in `scripts/run-specs.sh::SPEC_LIVE_PATHS` only after their replacements are green.

Completed under March on 2026-08-04, as March ticket 652. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/652-convert-clingen-cspec-and-erepo-live-assertions-to-receipt-backed-contracts
