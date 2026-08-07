---
flow: build
priority: 7
---
# Reformulation-loop breaker via session keyword-overlap detection

Agents reformulate `biomcp search article` 3+ times with overlapping keywords, burning turns and rarely improving results. 009 deep dive: 24 of 72 stopped-early tasks loop this way (33%). Guidance text "change strategy after 2 searches" doesn't stop it. Examples from the dive: `Oncotype DX review → Oncotype DX DCIS → Oncotype DX colon → Oncotype DX Genomic Prostate Score`; `mTOR abbreviation → what is mTOR → mammalian target of rapamycin → mechanistic target of rapamycin → mTOR signaling Laplante Sabatini 2012`.

Completed under March on 2026-04-21, as March ticket 278. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/278-reformulation-loop-breaker-via-session-keyword-overlap-detection
