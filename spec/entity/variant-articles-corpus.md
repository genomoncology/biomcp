# Frozen Seven-Variant Article Recall

The seven-variant recall gate replays BioMCP's real, receipted provider corpus.
It makes no public requests and needs no credentials. The fixture validates all
twelve declared landmarks, including the three honestly observed absences, and
runs the exact Europe PMC request plans through the production client in both
JSON and compact Markdown modes. PubMed's retained request plans and bodies run
through its production decoder in the Rust test suite.

## Seven-variant captured recall

<!-- mustmatch-lint: skip -->

Unknown requests are refused instead of receiving the nearest fixture. The
gate also preserves the captured positive, empty, degraded, and not-attempted
states; the bare protein panel correctly makes no CAR or LDH request.

```bash run id=variant-article-corpus-canary exit=0 timeout=180
bash ../fixtures/run-variant-articles-corpus-canary.sh ../..
```

```json expect=variant-article-corpus-canary contains
{
  "reference_recall_at_least_9_of_12": true,
  "variant_coverage_at_least_6_of_7": true,
  "mlh1_family_pmids_present": true,
  "route_specific_pmids_present_for_expected_variants": true,
  "expected_pmid_route_diagnostics_are_binary_attributed": true,
  "production_cli_consumed_exact_europepmc_captures": true,
  "compact_json_and_markdown_rendering_preserve_landmarks": true,
  "strict_unknown_route_rejected": true,
  "terminal_states_and_work_are_pinned_by_corpus_map": true
}
```
