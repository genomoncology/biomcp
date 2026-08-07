---
base: 8e3a913cfaad4a6b8effa08096ca802cdfa3d80e
head: cd5734a5f563e9ed38a1597eed9350634c917e4a
---
`src/cli/search_all.rs` is 2,984 lines and currently mixes cross-entity planning, per-section dispatch, follow-up link generation, value formatting/refinement, and a 1,000+ line inline test block. That makes the most coupled CLI helper surface hard to review and keeps the `src/cli/` 700-line cap as policy instead of a ratchet. The render layer also imports `crate::cli::search_all::SearchAllResults`, so this refactor must keep the stable module path while shrinking the implementation into review-sized files.

Imported from March ticket 319. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/319-decompose-search-all-rs-into-src-cli-search-all-submodules
