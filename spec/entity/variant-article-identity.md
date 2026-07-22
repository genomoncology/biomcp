# Frozen variant-article identity gate

Variant-article retrieval can find a paper through a query alias without proving
that the paper contains the requested gene and allele. This routine gate uses
frozen provider responses for the unchanged G5 v2 request panel, so release
proof measures captured evidence rather than live-provider availability.

## Verified positives survive collision filtering and pagination

<!-- mustmatch-lint: skip -->

The frozen panel retains its four independently linked positive papers, rejects
known wrong-gene or wrong-allele collisions, and records the three intentionally
unverified identities rather than silently treating them as successes. Filtering
confirmed observations before ranking and pagination keeps the APC positive
visible even when collision rows arrive first.

```bash run id=frozen-variant-article-identity exit=0
bash ../fixtures/run-variant-article-identity-fixture.sh ../..
```

```json expect=frozen-variant-article-identity contains
{
  "frozen_positive_statuses": {
    "apc": true,
    "atm": true,
    "palb2": true,
    "mlh1": true
  },
  "collision_pmids_never_confirmed": true,
  "intentional_unverified": {
    "brca1": true,
    "pten": true,
    "tp53": true
  },
  "conflicting_observation": true,
  "outage_is_incomplete": true,
  "confirmed_page_filters_before_limit": true,
  "audit_versions_and_canonical_subsets": true
}
```

The outage response is not a completed negative search: it remains incomplete,
truncated, and without a total. The report also requires versioned verifier and
provider-template facts plus canonical hashes of the clinically relevant
response and captured-content subsets, so citation ordering or count changes do
not become release criteria.
