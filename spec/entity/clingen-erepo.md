# Frozen ClinGen ERepo expert assertions

ClinGen's Evidence Repository records expert-panel assertion facts for a ClinGen
Allele identifier (CAid). BioMCP reports those facts without applying ACMG/AMP
rules or treating defaults and prose as evidence that a panel applied a strength.

## Frozen ERepo summaries, detail, and typed MCP preserve source facts

<!-- mustmatch-lint: skip -->

The frozen provider capture covers an APC summary with a plain `PS4` source token,
a structured `Strong` default, and comment text mentioning `PS4_VeryStrong`; those
are three distinct facts. It also covers explicit met and unmet criteria, PTEN's
missing `unMetCodes` coverage, an exact-search healthy miss, selected versioned
detail with citation locators, deterministic assertion selection, and equivalent
CLI/MCP batch output. The compact report keeps the executable documentation
focused on user-visible evidence truth rather than volatile provider payloads.

The summary-selected `--detail` workflow must consume its source plan and replay
receipt-admitted summary and detail bytes through the public CLI. The provider's
returned `@id` remains a source fact: its host need not match the local fixture
origin, while its selected assertion/version path must remain intact.

```bash run id=clingen-erepo-frozen exit=0
bash ../fixtures/run-clingen-erepo-fixture.sh ../..
```

```json expect=clingen-erepo-frozen contains
{
  "plain_cli_reports_summary": true,
  "empty_caid_resolves_its_car_gene": true,
  "empty_caid_with_no_car_gene_keeps_the_original_message": true,
  "unavailable_car_gene_lookup_keeps_the_original_message": true,
  "apc_summary_preserves_source_facts": true,
  "plain_ps4_has_no_explicit_strength": true,
  "default_strength_is_not_applied_strength": true,
  "comment_strength_is_not_applied_strength": true,
  "met_and_unmet_are_independent": true,
  "missing_unmet_coverage_is_not_empty": true,
  "healthy_exact_miss_is_empty_and_complete": true,
  "assertions_are_uuid_then_version_ordered": true,
  "multiple_assertions_require_explicit_selection": true,
  "selected_detail_keeps_version_and_citation_locator": true,
  "batch_preserves_order_and_duplicates": true,
  "cli_and_mcp_have_same_contract": true,
  "gene_page_is_bounded_and_truthful": true,
  "gene_second_page_is_reachable": true,
  "gene_cli_and_mcp_have_same_contract": true,
  "summary_and_detail_bounds_are_reported": true,
  "detail_cli_consumes_selected_source_plan": true,
  "receipted_summary_and_detail_drive_cli": true,
  "provider_at_id_is_preserved_in_detail": true
}
```
