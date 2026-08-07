---
flow: build
priority: 10
---
# Preserve article full-text and asset resolver ladder outcomes with bounds and tests

The article full-text/asset resolver ladder collapses source failure into confident absence. `src/entities/article/fulltext.rs` maps HTTP failure, body overflow, and HTML/PDF conversion failure to the same `Miss` state as a healthy absence; if no later rung succeeds, the public card states the consulted sources returned no full text, erasing whether retrieval actually completed (reproduced, high severity, at HEAD `e56630be`; the same contract class ticket 402 intended to harden). Separately, PMC OA archive expansion is unbounded, and the PMC OA→Europe PMC→Figshare asset resolver lacks deterministic coverage that each failure kind survives as `source_unavailable` rather than `not_found`.

Completed under March on 2026-07-17, as March ticket 582. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/582-preserve-article-full-text-and-asset-resolver-ladder-outcomes-with-bounds-and-tests
