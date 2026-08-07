---
base: 45471b7eb846bb3356d585fa02782c60760b54ba
head: bf19977a7b1fac072d9198764a965f0778498e79
---
`biomcp search trial --intervention` passes the user's drug name directly to ClinicalTrials.gov's `intr` parameter, which does literal substring matching against each trial's `InterventionName`. Different sponsors label the same drug differently — Revolution Medicines uses the generic name `daraxonrasib`, but Tango Therapeutics, Amgen, and Bristol Myers Squibb each use the sponsor code `RMC-6236` in their combination-trial listings. As a result:

Imported from March ticket 198. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/198-expand-trial-search-with-drug-alias-union
