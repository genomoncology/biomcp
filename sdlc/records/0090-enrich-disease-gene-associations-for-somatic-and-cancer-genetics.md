---
base: 0513ae9156ecaf2e1acac90c9b079581ecef8e79
head: 5288a57eb11365946b3cc643a38dd429d7f91feb
---
BioASQ call-chain optimization (research/009) revealed that `get disease ... genes` has excellent coverage for germline/Mendelian diseases (via OMIM) but near-zero coverage for somatic/cancer genetics. This makes the disease entity unreliable for cancer-related questions, forcing agents to fall back to article search every time — adding 4-8 unnecessary calls per question.

Imported from March ticket 090. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/090-enrich-disease-gene-associations-for-somatic-and-cancer-genetics
