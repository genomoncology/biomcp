---
flow: build
priority: 6
---

# Wire DDInter local data so drug interactions are answerable

Health check (2026-08-28): `DDInter local data ... not configured (default
path)`. The `biomcp ddinter` management command and the "drug interactions"
helper plus `get drug` interactions section exist but have no data behind
them. The EMA local-data pattern (download, local store, health line)
already works in this codebase; DDInter should follow it exactly.

## Done when

- `biomcp ddinter` fetches/refreshes the interaction dataset into the local
  store and reports availability in the health check the way EMA does.
- `get drug interactions <name>` and the drug-card interactions section
  populate from the local data with citations to DDInter records.
- The download is scripted and repeatable (the 188-experiment lesson: no
  manual steps that a fresh workstation cannot replay).
- Documented license/attribution for DDInter data lands with the feature.

Filed as build: authored integration work, no red to reproduce; suite green
as of 2026-08-27.
