---
flow: build
priority: 5
---
# Add renderer envelope fixture contracts for request reset

What is IN scope: - Renderer/envelope tests for disease, discover, article, and variant fixture models - `src/render/markdown/*`, `src/render/json/*`, and provenance helpers only where needed for testability or minimal ownership cleanup - `src/entities/discover.rs`, `src/entities/disease/*`, `src/entities/article/*`, and `src/entities/variant/*` only where needed to expose fixture result models or avoid markdown-specific dependencies in new request-contract paths - JSON `_meta.next_commands`, `_meta.source_status`, evidence/provenance, and markdown structural anchors

Completed under March on 2026-05-23, as March ticket 377. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/377-add-renderer-envelope-fixture-contracts-for-request-reset
