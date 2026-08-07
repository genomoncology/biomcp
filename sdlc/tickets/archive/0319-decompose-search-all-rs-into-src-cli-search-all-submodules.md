---
flow: build
priority: 8
---
# Decompose search_all.rs into src/cli/search_all/ submodules

`src/cli/search_all.rs` is 2,984 lines and currently mixes cross-entity planning, per-section dispatch, follow-up link generation, value formatting/refinement, and a 1,000+ line inline test block. That makes the most coupled CLI helper surface hard to review and keeps the `src/cli/` 700-line cap as policy instead of a ratchet. The render layer also imports `crate::cli::search_all::SearchAllResults`, so this refactor must keep the stable module path while shrinking the implementation into review-sized files.

Completed under March on 2026-04-26, as March ticket 319. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/319-decompose-search-all-rs-into-src-cli-search-all-submodules
