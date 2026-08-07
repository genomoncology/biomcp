---
base: 7dea706c0df09d3add7562700d6281844b4764e1
head: 17f02dbb24c99ee4d4ab6ee14d7f9266972af15e
---
`src/entities/article.rs` is 9,251 lines — the largest file in the codebase by far. It handles seven distinct concerns: types/config, backend planning, query construction, per-backend search legs, merge/dedup/cap, ranking/scoring, enrichment, federated orchestration, article detail, and 5,343 lines of tests.

Imported from March ticket 170. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/170-decompose-article-rs-into-article-submodule
