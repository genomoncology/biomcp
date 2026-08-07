---
base: f09e175e7ff96996a0a710731f116dd69d57fbc4
head: 5629001d0297ba619e09f7254f6030d33e56169b
---
Current `_meta.suggestions[]` in BioMCP are one-hop cross-entity pivots. They do not encode the known-efficient 3–4-step ladder for recurring question shapes. The 009 walkthrough corpus shows the same question shapes burning 8–20 calls today (warfarin-pharmacogene 15, disease-locus-mapping 20, mutation-catalog 8–9) because the agent has to rediscover the sequence each time. Skill text alone hasn't fixed this — how-to discovery rate is 0/192 tasks.

Imported from March ticket 282. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/282-workflow-ladder-emission-and-sidecar-schema-for-first-call-hateoas
