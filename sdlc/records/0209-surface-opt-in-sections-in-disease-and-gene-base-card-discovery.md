---
base: 7951cab61132db99d9c3a8948d58e68f93a8ece4
head: 5e211a2d7e908606b18eaa75b63619ca72f7a829
---
`get disease melanoma` and `get gene BRAF` base cards list valid follow-up sections in the "More:" block but omit `survival` (SEER Explorer) and `funding` (NIH Reporter). A user who arrives at the base card has no way to discover these opt-in sections without reading `biomcp list disease` or `biomcp get disease --help`. The sections are correctly excluded from `all` but have zero discovery surface from the natural user path.

Imported from March ticket 209. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/209-surface-opt-in-sections-in-disease-and-gene-base-card-discovery
