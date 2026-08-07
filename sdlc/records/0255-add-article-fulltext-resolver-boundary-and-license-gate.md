---
base: cd54e114a3079fe24ff8fc2a4ca8a3835979338f
head: f7a8999ed5106ea24ef87fc55b085bf7ad5740b6
---
Survey issues 1, 2, 5, and 6 block safe adoption of the spike. Article fulltext resolution is still embedded in `src/entities/article/detail.rs`, the renderer and provenance layer assume every saved file is `PMC OA`, the cache key assumes one extractor family, and the repo has no enforced license allowlist for the new HTML/PDF crates. Before any new extractor ships, BioMCP needs a dedicated article-fulltext boundary around the existing JATS path plus a `cargo deny` gate that makes the license policy executable.

Imported from March ticket 255. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/255-add-article-fulltext-resolver-boundary-and-license-gate
