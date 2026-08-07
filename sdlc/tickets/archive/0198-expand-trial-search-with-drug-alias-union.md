---
flow: build
priority: 6
---
# Expand trial search with drug alias union

`biomcp search trial --intervention` passes the user's drug name directly to ClinicalTrials.gov's `intr` parameter, which does literal substring matching against each trial's `InterventionName`. Different sponsors label the same drug differently — Revolution Medicines uses the generic name `daraxonrasib`, but Tango Therapeutics, Amgen, and Bristol Myers Squibb each use the sponsor code `RMC-6236` in their combination-trial listings. As a result:

Completed under March on 2026-04-15, as March ticket 198. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/198-expand-trial-search-with-drug-alias-union
