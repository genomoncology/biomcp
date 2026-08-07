---
flow: quickfix
priority: 9
---
# Resolve gene all-section warm-budget xdist flake before release cut

`spec/entity/gene.md::All-Section Warm Budget` (the assertion shipped by ticket 316) trips under 16-worker xdist parallelism: warm BRCA1 `get gene all` passes in isolation around 6–7s but spikes to ~10s under contention, exceeding the 7000ms ceiling. Any PR run of `make spec-pr` can fail non-deterministically while the runtime is healthy. The 327 release-readiness review flagged this as release-blocking before the v0.8.22 release cut.

Completed under March on 2026-04-27, as March ticket 328. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/328-resolve-gene-all-section-warm-budget-xdist-flake-before-release-cut
