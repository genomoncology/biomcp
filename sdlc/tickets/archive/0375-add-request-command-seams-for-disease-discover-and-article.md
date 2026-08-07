---
flow: build
priority: 8
---
# Add request-command seams for disease discover and article

What is IN scope: - `src/cli/discover.rs` - `src/entities/discover.rs` - `src/cli/disease/dispatch.rs` - `src/entities/disease/search.rs` - `src/entities/disease/fallback.rs` - `src/cli/article/dispatch.rs` - `src/entities/article/planner.rs` tests only as needed to expose pre-execution request-command assertions - Unit tests for normalized disease/discover/article request values and fallback/routing decisions

Completed under March on 2026-05-23, as March ticket 375. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/375-add-request-command-seams-for-disease-discover-and-article
