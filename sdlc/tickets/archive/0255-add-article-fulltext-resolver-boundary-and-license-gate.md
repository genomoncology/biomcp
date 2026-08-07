---
flow: build
priority: 8
---
# Add article fulltext resolver boundary and license gate

Survey issues 1, 2, 5, and 6 block safe adoption of the spike. Article fulltext resolution is still embedded in `src/entities/article/detail.rs`, the renderer and provenance layer assume every saved file is `PMC OA`, the cache key assumes one extractor family, and the repo has no enforced license allowlist for the new HTML/PDF crates. Before any new extractor ships, BioMCP needs a dedicated article-fulltext boundary around the existing JATS path plus a `cargo deny` gate that makes the license policy executable.

Completed under March on 2026-04-20, as March ticket 255. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/255-add-article-fulltext-resolver-boundary-and-license-gate
