---
flow: build
priority: 28
---

# Give the PMC3040717 stored fixture its proof-of-work routes

The stored-source fixture server behind the article assets flow has no
handler for either `/articles/instance/3040717/bin/` supplementary URL, so
`get article 20516115 assets` reports `healthy_absent` for both PMC3040717
supplementary files instead of the `pmc_proof_of_work` outcome ticket 1045
established. A separate PMC123466 fixture does exercise the proof-of-work
outcome, so the behavior is covered elsewhere but not for the article whose
own fixture HTML points at those exact bin URLs. Verified on 2026-08-23:
the fixture setup serves the PMC123466 bin route and has no 3040717 bin
route, while the stored PMC3040717 supplementary-tables HTML links only to
`/articles/instance/3040717/bin/...` targets.

## Done when

- The stored fixture serves routes for the supplementary bin URLs that the
  PMC3040717 fixture HTML links, so the stored-source assets flow reports
  the proof-of-work outcome for both entries.
- The outcome is pinned by contract coverage that runs against the stored
  fixture — outside ticket 1045's HTML-only parsing scope, exercising the
  stored-source path end to end.

Filed from `sdlc/issues/2026-08-23-pmc3040717-fixture-outcome.md`, which
records the 2026-08-23 observation this ticket completes.
