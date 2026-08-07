---
flow: build
priority: 5
---
# Enrich disease-gene associations for somatic and cancer genetics

BioASQ call-chain optimization (research/009) revealed that `get disease ... genes` has excellent coverage for germline/Mendelian diseases (via OMIM) but near-zero coverage for somatic/cancer genetics. This makes the disease entity unreliable for cancer-related questions, forcing agents to fall back to article search every time — adding 4-8 unnecessary calls per question.

Completed under March on 2026-03-31, as March ticket 090. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/090-enrich-disease-gene-associations-for-somatic-and-cancer-genetics
