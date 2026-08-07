---
flow: build
priority: 6
---
# Workflow ladder emission and sidecar schema for first-call HATEOAS

Current `_meta.suggestions[]` in BioMCP are one-hop cross-entity pivots. They do not encode the known-efficient 3–4-step ladder for recurring question shapes. The 009 walkthrough corpus shows the same question shapes burning 8–20 calls today (warfarin-pharmacogene 15, disease-locus-mapping 20, mutation-catalog 8–9) because the agent has to rediscover the sequence each time. Skill text alone hasn't fixed this — how-to discovery rate is 0/192 tasks.

Completed under March on 2026-04-22, as March ticket 282. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/282-workflow-ladder-emission-and-sidecar-schema-for-first-call-hateoas
