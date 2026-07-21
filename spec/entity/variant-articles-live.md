# Live Variant-Article Recall

These operator canaries check exact-variant literature discovery and recall
against real providers. They do not treat extra papers as false positives or
clinical evidence labels.

## Article searches expose the exact-variant helper

<!-- mustmatch-lint: skip -->

An exact gene-and-protein-change keyword remains an ordinary article search. Its
JSON follow-ups also point to the exact-route variant literature helper, even
when the provider currently returns no papers, so an agent does not have to
invent command grammar.

```bash run id=exact-variant-article-search exit=0 timeout=180
../../tools/biomcp-ci --no-cache --json search article -k "MSH2 p.L341P" --source pubtator --limit 1
```

```json expect=exact-variant-article-search contains
{
  "_meta": {
    "next_commands": ["biomcp variant articles \"MSH2 p.L341P\""]
  }
}
```

## Seven-Variant Recall Canary

<!-- mustmatch-lint: skip -->

The union route should recover the predeclared readiness threshold, cover most
panel variants, preserve the two MLH1 family papers, and retain every PMID for
the same variant where an individual route had already demonstrated it, with
route provenance. Provider drift must be reported
rather than hidden by weakening the routine fixture contract.

```bash run id=variant-article-live-canary exit=0 timeout=180
bash ../fixtures/run-variant-articles-live-canary.sh ../..
```

```json expect=variant-article-live-canary contains
{
  "reference_recall_at_least_9_of_12": true,
  "variant_coverage_at_least_6_of_7": true,
  "mlh1_family_pmids_present": true,
  "route_specific_pmids_present_for_expected_variants": true
}
```
