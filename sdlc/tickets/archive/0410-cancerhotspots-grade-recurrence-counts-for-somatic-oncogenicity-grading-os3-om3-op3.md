---
flow: build
priority: 5
---
# Cancerhotspots-grade recurrence counts for somatic oncogenicity grading (OS3/OM3/OP3)

Today `biomcp get variant <id> all` exposes recurrence only as `cancer_frequencies` — per-cancer `sample_count` from a **single locked cBioPortal study** (`msk_impact_2017`, ~10–12K samples). It carries **no position-level count at all**, so OS3's position number cannot be computed, and the per-AA `sample_count` it does return is the wrong cohort's number. A variant-classification agent on frozen biomcp therefore **cannot grade the hotspot tier**, regardless of the rest of the pipeline. (Verified 2026-06-10 in the evidence team's variant-classification Evolver work.)

Completed under March on 2026-06-10, as March ticket 410. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/410-cancerhotspots-grade-recurrence-counts-for-somatic-oncogenicity-grading-os3-om3-op3
