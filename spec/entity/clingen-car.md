# Frozen ClinGen Allele Registry contract

ClinGen Allele Registry (CAR) normalizes supported versioned RefSeq HGVS values
without inferring biological equivalence. This routine gate uses a bounded local
CAR service so the public CLI and typed MCP transport have deterministic proof;
the separate live-provider diagnostic remains under `make verify`.

## Frozen CAR CLI and typed MCP contract

The frozen panel retains its returned CAids, source-labelled aliases, request
provenance, grammar boundary, GET/POST transport, batch order and duplicate
preservation, response-cardinality handling, and all-provider ordering. The
fixture calls the public CLI and typed MCP tool, rather than source helpers.

```bash
bash ../fixtures/run-clingen-car-fixture.sh ../.. | mustmatch like '"cli_and_typed_mcp_parity": true
"frozen_identity_panel": true
"request_templates": true
"batch_order_and_duplicates": true
"batch_cardinality_mismatch_is_incomplete": true
"grammar_and_bounds": true
"version_provenance": true
"normalize_all_order_and_outage_isolation": true'
```

APC GRCh37 returning `CA015543` and `NM_000038.6:c.847C>G` returning
`CA16023172` are CAR lookup facts. They do not claim equivalence or liftover.

## Terminal source facts and bounded external IDs

A minimal CAR blank node is an exhaustive miss, but a present malformed
projected source fact is indeterminate rather than a healthy negative. Returned
external IDs retain each source's breadth even when rendering is capped: dbSNP
precedes ClinVar, each source renders at most eight numeric IDs, and the scalar
source count is the combined full distinct count.

```bash
bash ../fixtures/run-clingen-car-fixture.sh ../.. | mustmatch like '"minimal_blank_node_is_exhaustive_not_found": true
"malformed_blank_node_is_indeterminate": true
"malformed_blank_node_has_no_credited_facts": true
"external_ids_have_independent_source_caps": true
"external_ids_report_full_distinct_source_count": true
"external_ids_report_truncation": true
"external_ids_are_numeric_and_source_ordered": true'
```
