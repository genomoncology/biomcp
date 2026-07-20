# Live Variant-Article Recall

This operator canary checks exact-variant literature recall against the factual,
non-exhaustive seven-variant panel that motivated the union workflow. It uses
real providers under `make verify`; extra papers are not treated as false
positives or clinical evidence labels.

## Seven-Variant Recall Canary

<!-- mustmatch-lint: skip -->

The union route should recover the predeclared readiness threshold, cover most
panel variants, preserve the two MLH1 family papers, and retain every PMID that
an individual route had already demonstrated. Provider drift must be reported
rather than hidden by weakening the routine fixture contract.

```bash run id=variant-article-live-canary exit=0 timeout=180
bash ../fixtures/run-variant-articles-live-canary.sh ../..
```

```json expect=variant-article-live-canary contains
{
  "reference_recall_at_least_9_of_12": true,
  "variant_coverage_at_least_6_of_7": true,
  "mlh1_family_pmids_present": true,
  "route_specific_pmids_present": true
}
```
