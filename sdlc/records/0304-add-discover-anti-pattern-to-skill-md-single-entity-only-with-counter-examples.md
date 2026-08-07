---
base: 5c0378b8267507c15f54368c45ef2181e059e682
head: 9c6f8c9facba5910791de7f5893e59e03ef0df27
---
Re-frame `biomcp discover` in SKILL.md as a single-entity free-text lookup, not a relational query tool. Adds two counter-examples (warfarin drug-interaction, MEF-2 gene-target) demonstrated to cut noisy `discover` calls by 74% in research 009 with no answer-quality regression in isolation.

Imported from March ticket 304. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/304-add-discover-anti-pattern-to-skill-md-single-entity-only-with-counter-examples
