---
flow: build
priority: 8
---
# cBioPortal: surface structural-variant/fusion data in study analytics (it is already on disk)

`biomcp study query --type mutations` returns SNVs/indels only and silently excludes gene fusions / structural variants. For RET and NTRK the *actionable* lesion is a fusion, so an agent reads NTRK as a ~5% point-mutation target when the real NTRK-fusion rate is ~0.2% — a confidently-wrong biological conclusion (flagged as the most consequential finding in the exp086 bundle).

Completed under March on 2026-06-16, as March ticket 423. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/423-cbioportal-surface-structural-variant-fusion-data-in-study-analytics-it-is-already-on-disk
