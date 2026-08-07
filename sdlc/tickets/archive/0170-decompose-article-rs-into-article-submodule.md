---
flow: build
priority: 1
---
# Decompose article.rs into article submodule

`src/entities/article.rs` is 9,251 lines — the largest file in the codebase by far. It handles seven distinct concerns: types/config, backend planning, query construction, per-backend search legs, merge/dedup/cap, ranking/scoring, enrichment, federated orchestration, article detail, and 5,343 lines of tests.

Completed under March on 2026-04-11, as March ticket 170. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/170-decompose-article-rs-into-article-submodule
