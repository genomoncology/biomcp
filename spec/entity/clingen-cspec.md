# Frozen ClinGen CSpec documents

ClinGen's Criteria Specification Registry publishes versioned VCEP source
specifications. BioMCP lists the exact provider resource IRIs, captures one selected
document before parsing it, and keeps raw source bytes available through a CLI-only
capture handle.

## Frozen manifest, captured document, and typed MCP page preserve one source capture

<!-- mustmatch-lint: skip -->

The frozen provider fixture contains BRAF's historical GN004 and current GN049
series plus ATM GN020. It proves that the full provider IRI remains distinct from
the display version, parsed criteria and raw output share one capture, and paging
or typed MCP access does not turn source text into an ACMG conclusion.

```bash run id=clingen-cspec-frozen exit=0
bash ../fixtures/run-clingen-cspec-fixture.sh ../..
```

```json expect=clingen-cspec-frozen contains
{
  "braf_keeps_gn004_and_gn049": true,
  "atm_full_iri_is_distinct_from_display_version": true,
  "same_capture_raw_sha256_and_length_match": true,
  "raw_document_does_not_refetch_cspec": true,
  "criteria_pages_are_provider_identity_ordered": true,
  "cli_and_mcp_capture_page_match": true,
  "criteria_do_not_claim_interpretation": true
}
```
