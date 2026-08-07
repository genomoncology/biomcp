---
base: 3d2f1fa27caf3b8980fd03764564bc64bc6de65a
head: 5d21fcf1041d9a0ec700c0bf283e1d400e31cd14
---
Today `biomcp get variant <id> all` exposes recurrence only as `cancer_frequencies` — per-cancer `sample_count` from a **single locked cBioPortal study** (`msk_impact_2017`, ~10–12K samples). It carries **no position-level count at all**, so OS3's position number cannot be computed, and the per-AA `sample_count` it does return is the wrong cohort's number. A variant-classification agent on frozen biomcp therefore **cannot grade the hotspot tier**, regardless of the rest of the pipeline. (Verified 2026-06-10 in the evidence team's variant-classification Evolver work.)

Imported from March ticket 410. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/410-cancerhotspots-grade-recurrence-counts-for-somatic-oncogenicity-grading-os3-om3-op3
