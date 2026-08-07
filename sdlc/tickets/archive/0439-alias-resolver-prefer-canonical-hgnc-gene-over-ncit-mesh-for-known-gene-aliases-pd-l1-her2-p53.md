---
flow: build
priority: 5
---
# Alias resolver: prefer canonical HGNC gene over NCIT/MESH for known gene aliases (PD-L1/HER2/P53)

Prefer the canonical HGNC gene for a known gene alias over NCIT/MESH drug/symptom entities (PD-L1 returns drugs, omits CD274); fold in HER2/P53 hint + alias-set audit. Verified high-severity data bug on 0.8.24.

Completed under March on 2026-06-24, as March ticket 439. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/439-alias-resolver-prefer-canonical-hgnc-gene-over-ncit-mesh-for-known-gene-aliases-pd-l1-her2-p53
