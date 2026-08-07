---
base: 43d6fb4e58266951a7fdc73a350e50c9678b0aa7
head: e5dba470eb6ffbb8e256a3c2718a5237f880107c
---
`biomcp study query --type mutations` returns SNVs/indels only and silently excludes gene fusions / structural variants. For RET and NTRK the *actionable* lesion is a fusion, so an agent reads NTRK as a ~5% point-mutation target when the real NTRK-fusion rate is ~0.2% — a confidently-wrong biological conclusion (flagged as the most consequential finding in the exp086 bundle).

Imported from March ticket 423. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/423-cbioportal-surface-structural-variant-fusion-data-in-study-analytics-it-is-already-on-disk
