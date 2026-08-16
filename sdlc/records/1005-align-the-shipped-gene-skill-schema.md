---
flow: quickfix
priority: 9
---

# Align the shipped gene skill schema

The CLI now emits `genomic_coordinates` as a typed object, while the shipped gene schema and BRAF example still describe a string. Define the outer field as object or null without making it required. The object has exactly `coordinate`, `genome_build`, `source`, and optional `provenance`; require the first three, reject additional nested properties, and make `provenance` a string when present rather than a nullable value. This matches the Rust serializer, which omits absent provenance and can omit the whole optional coordinate field.

Update the BRAF example from the captured MyGene fixture with coordinate `7:140719327-140925199 (strand: -1)`, build `GRCh38`, source `MyGene.info`, and provenance `MyGene.info provider default`. The owning public files are `skills/schemas/gene.json` and `skills/examples/get-gene-BRAF.json`.

Add a fail-closed CLI-to-schema assertion to `spec/entity/gene.md`, which already runs with the captured MyGene provider fixture in the routine offline lane. It must validate the actual `biomcp --json get gene BRAF` payload, not only the checked-in example. `scripts/validate-skills.sh` may retain its static example check and its existing optional live tier unchanged; do not start a public provider or add a fixture dependency to `spec/surface/skills.md` or `spec-contracts`.
