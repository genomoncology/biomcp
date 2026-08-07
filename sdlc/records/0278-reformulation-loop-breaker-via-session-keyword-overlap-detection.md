---
base: 6255001a8c4b1e13da205dc275042964687ff0f3
head: 5615e8a04bdded6f392b77ec9348ec96cba196d1
---
Agents reformulate `biomcp search article` 3+ times with overlapping keywords, burning turns and rarely improving results. 009 deep dive: 24 of 72 stopped-early tasks loop this way (33%). Guidance text "change strategy after 2 searches" doesn't stop it. Examples from the dive: `Oncotype DX review → Oncotype DX DCIS → Oncotype DX colon → Oncotype DX Genomic Prostate Score`; `mTOR abbreviation → what is mTOR → mammalian target of rapamycin → mechanistic target of rapamycin → mTOR signaling Laplante Sabatini 2012`.

Imported from March ticket 278. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/278-reformulation-loop-breaker-via-session-keyword-overlap-detection
