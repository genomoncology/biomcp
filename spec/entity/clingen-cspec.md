# Frozen ClinGen CSpec capture contract

ClinGen's Criteria Specification Registry provides versioned source documents. BioMCP first lists the exact provider resource IRIs, then captures a selected document so parsed pages and CLI raw retrieval retain one reproducible source.

## Manifest selection preserves source identity

The routine fixture contains the named source series, including BRAF's separate GN004 and GN049 documents. An exact ATM IRI is not its shorter display version, and parsed rows retain source order, bounded reference URLs, null disease, and a page-independent semantic source digest.

```bash run id=clingen-cspec-capture-contract exit=0
bash ../fixtures/run-clingen-cspec-fixture.sh ../..
```

```text expect=clingen-cspec-capture-contract contains
"all_named_gene_series_are_available": true
"braf_keeps_gn004_and_gn049": true
"atm_uses_literal_full_iri_not_display_version": true
"literal_selector_returns_matching_gene_and_specification": true
"json_switches_cspec_manifest_and_page_output": true
"criteria_are_deterministic_and_paged": true
"supported_reference_objects_preserve_ordered_deduplicated_urls": true
"disease_is_null": true
"semantic_subset_is_page_independent": true
```

## Receipt-backed captured pages keep the selected identity

A capture binds the selected ATM document and gene rather than trusting a later label. Its raw bytes match reported provenance, raw retrieval does not fetch the provider again, and the typed MCP page is the same public projection as the CLI page. The routine fixture must replay receipt-admitted ClinGen manifest and version-page bytes through the shipped command. This proves the manifest request selects the exact resource IRI, the selected page remains capture-backed, and paging preserves provider criterion order without making current provider availability part of the routine gate.

```text expect=clingen-cspec-capture-contract contains
"capture_binds_requested_gene_and_selected_iri": true
"cli_capture_page_matches_typed_mcp": true
"caller_gene_cannot_relabel_capture": true
"missing_capture_is_capture_unavailable": true
"raw_bytes_match_reported_sha256_and_length": true
"raw_read_does_not_refetch": true
"receipt_backed_manifest_plan_is_consumed": true
"receipted_manifest_and_version_page_drive_cli": true
"paged_capture_keeps_provider_criterion_order": true
"pten_attachment_manifest_is_bounded_metadata_only": true
"normal_criteria_reports_attachment_count": true
"attachment_capture_reuse_does_not_refetch": true
"attachment_cli_and_mcp_match": true
```
