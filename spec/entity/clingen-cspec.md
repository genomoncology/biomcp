# Frozen ClinGen CSpec capture contract

ClinGen's Criteria Specification Registry provides versioned source documents. BioMCP first lists the exact provider resource IRIs, then captures a selected document so parsed pages and CLI raw retrieval retain one reproducible source.

## Manifest selection preserves source identity

The routine fixture contains the named source series, including BRAF's separate GN004 and GN049 documents. An exact ATM IRI is not its shorter display version, and parsed rows retain source order, bounded reference URLs, null disease, and a page-independent semantic source digest.

```bash
bash ../fixtures/run-clingen-cspec-fixture.sh ../.. | mustmatch like '"all_named_gene_series_are_available": true
"braf_keeps_gn004_and_gn049": true
"atm_uses_literal_full_iri_not_display_version": true
"literal_selector_returns_matching_gene_and_specification": true
"criteria_are_deterministic_and_paged": true
"supported_reference_objects_preserve_ordered_deduplicated_urls": true
"disease_is_null": true
"semantic_subset_is_page_independent": true'
```

## Captured pages keep the selected identity

A capture binds the selected ATM document and gene rather than trusting a later label. Its raw bytes match reported provenance, raw retrieval does not fetch the provider again, and the typed MCP page is the same public projection as the CLI page.

```bash
bash ../fixtures/run-clingen-cspec-fixture.sh ../.. | mustmatch like '"capture_binds_requested_gene_and_selected_iri": true
"cli_capture_page_matches_typed_mcp": true
"caller_gene_cannot_relabel_capture": true
"raw_bytes_match_reported_sha256_and_length": true
"raw_read_does_not_refetch": true'
```

## Unavailable handles have a recovery-safe public code

An unknown capture handle is unavailable rather than invalid caller syntax. Integrity damage, expiration, eviction, binding conflicts, malformed provider rows, and capacity limits are defensive/native tests because proving them here would require deliberately damaging the environment or source.

```bash
bash ../fixtures/run-clingen-cspec-fixture.sh ../.. | mustmatch like '"missing_capture_is_capture_unavailable": true'
```
