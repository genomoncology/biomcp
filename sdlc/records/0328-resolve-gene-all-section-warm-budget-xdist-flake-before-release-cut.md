---
base: 42dfe59f80076a1d7805e0ef176ed1d60287f14f
head: feb19d8bab83624f43e3b16df8c3faaf11a26ef0
---
`spec/entity/gene.md::All-Section Warm Budget` (the assertion shipped by ticket 316) trips under 16-worker xdist parallelism: warm BRCA1 `get gene all` passes in isolation around 6–7s but spikes to ~10s under contention, exceeding the 7000ms ceiling. Any PR run of `make spec-pr` can fail non-deterministically while the runtime is healthy. The 327 release-readiness review flagged this as release-blocking before the v0.8.22 release cut.

Imported from March ticket 328. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/328-resolve-gene-all-section-warm-budget-xdist-flake-before-release-cut
