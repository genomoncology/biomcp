---
base: 3dc693dc8a97bec2cc86463fe655e782d4e6150c
head: db5530db1a6489438f0ccc34b03e6d960dfbaae1
---
Variant→article retrieval lost its deterministic recall in the Python→Rust rewrite. This is a **regression**, not a missing feature.

Imported from March ticket 426. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/426-restore-variant-concept-to-article-entity-autocomplete-regression-from-python-v0-7-3
