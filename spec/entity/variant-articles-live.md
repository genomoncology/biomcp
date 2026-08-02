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

## G5 v2 live readiness diagnostic

<!-- mustmatch-lint: skip -->

This focused live probe diagnoses provider state; the frozen routine contract in
`variant-article-identity.md` owns verified positives, collision rejection,
pagination, and outage truthfulness. When this probe runs inside the one
preflighted authoritative verify, its unchanged identity facts are hard release
gate evidence rather than an optional diagnostic. This probe checks that every
unchanged G5 v2 request still resolves, executes an exact route with a
route-tied literal alias, and reports source coverage plus terminal state
against real providers. Its debug plan also reconciles each reported work
allocation with the item budget and the recorded strict, exact, and verification
provider calls, so the diagnostic cannot claim unperformed work or hide work
outside its parent budget. `incomplete_results` reports only incomplete items with
recorded unperformed internal work; ordinary page pagination and recorded provider
degradation remain visible in each item's terminal/source status instead of being
misreported as an internal budget failure. `route_status_contradictions` is empty
only when a provider's source status agrees with its own recorded route calls and
every stop explanation names a route in `stopped_routes`. ATM and PALB2 also submit independent versioned RefSeq
transcript/coding and genomic identities to the real CAR: each must expose a
complete, exhaustive confirmed CAid with auditable transcript and genomic
observations. CAR evidence is additive to the article result; this probe does
not infer liftover or make it a clinical claim.

```bash run id=g5-v2-identity-live-canary exit=0 timeout=180
bash ../fixtures/run-g5-v2-identity-live-canary.sh ../..
```

```json expect=g5-v2-identity-live-canary contains
{
  "identity_readiness": {
    "expected_request_ids": true,
    "all_resolved": true,
    "all_have_exact_route": true,
    "all_have_route_tied_alias": true,
    "all_have_source_status": true,
    "all_have_terminal_state": true,
    "work_allocation_is_consistent_with_budgets_and_recorded_calls": true,
    "authoritative_verify_treats_g5_as_hard": true
  },
  "identity_diagnostics": {
    "schema_parse_failures": [],
    "missing_available_positives": [],
    "incomplete_results": [],
    "internal_misattributions": [],
    "route_status_contradictions": [],
    "missing_canonical_equivalence": []
  }
}
```

## Provider-specific strict query provenance

<!-- mustmatch-lint: skip -->

This live canary checks the request plan rather than article recall. It uses the
frozen coding collisions to ensure that each strict provider query keeps the gene
attached to its alias, retains a discovery request, and labels aliases as retrieval
inputs rather than observed article evidence.

```bash run id=provider-strict-query-live-canary exit=0 timeout=180
bash ../fixtures/run-variant-article-strict-live-canary.sh ../..
```

```json expect=provider-strict-query-live-canary contains
{
  "all_strict_templates_exact": true,
  "brca1_aliases_remain_distinct": true,
  "discovery_route_retained": true,
  "strict_route_executed": true,
  "provenance_uses_query_aliases_only": true
}
```

## Seven-Variant Recall Canary

<!-- mustmatch-lint: skip -->

The union route should recover the predeclared readiness threshold, cover most
panel variants, preserve the two MLH1 family papers, and retain every PMID for
the same variant where an individual route had already demonstrated it, with
route provenance. Its JSON attributes every expected PMID to binary-emitted
route-stage evidence, so provider absence cannot be confused with a BioMCP
pipeline loss. Provider drift must be reported rather than hidden by weakening
the routine fixture contract.

```bash run id=variant-article-live-canary exit=0 timeout=180
bash ../fixtures/run-variant-articles-live-canary.sh ../..
```

```json expect=variant-article-live-canary contains
{
  "reference_recall_at_least_9_of_12": true,
  "variant_coverage_at_least_6_of_7": true,
  "mlh1_family_pmids_present": true,
  "route_specific_pmids_present_for_expected_variants": true,
  "expected_pmid_route_diagnostics_are_binary_attributed": true
}
```
