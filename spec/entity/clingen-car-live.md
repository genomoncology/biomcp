# Live ClinGen Allele Registry normalization

ClinGen Allele Registry (CAR) supplies a canonical CAid for supported HGVS
identities. This diagnostic calls the public read-only registry; run it with
`make verify`, not the offline routine spec gate.

## Normalize a versioned transcript HGVS through the public CAR

A recognized transcript coding HGVS returns a source-labelled resolved item. Its
CAid and aliases are provider facts, while the submitted input remains visible
as the query that was resolved.

```bash
../../tools/biomcp-ci --json variant normalize car 'NM_000546.6:c.215C>G' \
  | jq '{input: .input, status: .status, exhaustive: .exhaustive, has_caid: (.caid | test("^CA[0-9]+$")), source: .source, query: .query, alias_collections: ([.genomic_aliases, .transcript_aliases, .protein_aliases, .external_ids] | all((.values | type) == "array" and (.source_count | type) == "number" and (.truncated | type) == "boolean")), provenance: {template_version: (.provenance.request_template_version | type), has_car_version: (.provenance | has("car_version"))}}' \
  | mustmatch like '{"input":"NM_000546.6:c.215C>G","status":"resolved","exhaustive":true,"has_caid":true,"source":"clingen_car","query":"NM_000546.6:c.215C>G","alias_collections":true,"provenance":{"template_version":"string","has_car_version":true}}'
```

## Retain CAR in the all-provider normalization order

The aggregate selector keeps its established transcript-only grammar, runs all
three providers, and presents their results in documented order rather than
network completion order.

```bash
../../tools/biomcp-ci --json variant normalize all 'NM_000546.6:c.215C>G' \
  | jq '[.services[] | .service]' \
  | mustmatch '["mutalyzer","variantvalidator","car"]'
```
