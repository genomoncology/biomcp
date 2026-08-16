---
flow: quickfix
priority: 9
---

# Align the shipped gene skill schema

The CLI now emits `genomic_coordinates` as a typed object, while the shipped gene schema and BRAF example still describe a string. Make the schema and example match the real nullable coordinate object and ensure a fixture-backed CLI payload is validated against that schema in the routine offline specification lane.

The owning public files are `skills/schemas/gene.json` and `skills/examples/get-gene-BRAF.json`. Deterministic coverage may be added to `spec/surface/skills.md`, `scripts/validate-skills.sh`, and their existing tests; live validation must remain optional rather than becoming a routine network dependency.
