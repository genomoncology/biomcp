---
flow: build
priority: 7
---
# Surface opt-in sections in disease and gene base card discovery

`get disease melanoma` and `get gene BRAF` base cards list valid follow-up sections in the "More:" block but omit `survival` (SEER Explorer) and `funding` (NIH Reporter). A user who arrives at the base card has no way to discover these opt-in sections without reading `biomcp list disease` or `biomcp get disease --help`. The sections are correctly excluded from `all` but have zero discovery surface from the natural user path.

Completed under March on 2026-04-15, as March ticket 209. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/209-surface-opt-in-sections-in-disease-and-gene-base-card-discovery
