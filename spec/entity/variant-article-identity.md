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
  "real_receipted_identity_anchor": true,
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
  "canonical_equivalence_is_additive": true,
  "canonical_observation_statuses_are_closed": true,
  "debug_plan_provider_statuses_are_closed": true,
  "confirmed_page_filters_before_limit": true,
  "audit_versions_and_canonical_subsets": true,
  "clingen_ldh": {
    "atm_exact_annotation_confirmed": true,
    "palb2_table_selector_confirmed": true,
    "empty_coverage_preserves_candidates": true
  },
  "deep_discovery_keeps_structured_braf_for_identity_verification": true,
  "identity_verification_is_bounded_to_visible_page": true,
  "debug_plan_records_discovery_and_verification_allocation": true,
  "candidate_route_trace_is_versioned_bounded_and_stage_attributed": true,
  "visible_results_have_candidate_route_receipts": true,
  "candidate_route_trace_keeps_filtered_observations": true,
  "candidate_route_trace_keeps_duplicate_route_observations": true,
  "candidate_route_trace_reports_omissions_and_retains_offset_visible_receipt": true,
  "typed_corresponding_gene_proof_is_pmid_bound": true,
  "document_identity_anomalies_are_incomplete_without_false_contradiction": true,
  "association_without_typed_linkage_is_unverified": true,
  "expected_pmid_aggregation_is_order_independent": true
}
```

The successful external identity anchor is the receipted TP53
`NM_000546.6:c.215C>G` chain through CAR `CA000072` and the ClinGen LDH table
annotation for `PMC8372092`. The fixture serves those byte-faithful responses
through the normal provider clients and checks both JSON and Markdown output.
The older opaque `CA900...` rows below remain explicitly synthetic tests of
collisions, ordering, bounded work, and failure handling; they are not evidence
about real variants or papers.

Canonical-equivalence observations and debug-plan provider rows are separate
machine interfaces, but both carry closed statuses. The frozen panel exercises
CAR observations and requested debug plans without making provider availability
a release criterion.

The outage response is not a completed negative search: it remains incomplete,
truncated, and without a total. The two eligible RefSeq requests also retain
MyVariant `not_found` while exposing a complete, auditable CAR sibling fact; the
fixture's opaque CAid tokens test local aggregation, not registry truth. The report also requires versioned verifier and
provider-template facts plus canonical hashes of the clinically relevant
response and captured-content subsets, so citation ordering or count changes do
not become release criteria.

## ClinGen LDH observations remain additive and bounded

ClinGen Linked Data Hub annotations are post-retrieval identity observations,
not article discovery. The frozen candidates prove that exact CAid/gene/PMCID
and selector links can confirm ATM and PALB2, including a PALB2 annotation that
also carries unrelated CAids. Empty LDH coverage leaves existing candidates
available without treating the absence as negative evidence.

## Deep discovery reserves identity verification for the requested page

A complete structured BRAF request supplies both `NM_004333.6:c.1799T>A` and
`NC_000007.13:g.140453136A>T`. The frozen PubMed response repeats one same-case
candidate deeply enough to consume an unreserved discovery budget. BioMCP must
leave work for the requested page's identity verification: `--confirmed-only`
returns the typed-PubTator-confirmed papers, never an unverified row, and keeps
complete CAR agreement for the two caller-supplied forms. Because discovery stops
before the provider's full result set, the response remains incomplete and
truncated rather than claiming exhaustive search. An ordinary request verifies
only its visible page, so a small page does not spend identity work on later
candidates; `--confirmed-only` retains its bounded pre-pagination lookahead to
find confirmed rows. Its debug plan records the discovery, exact-lexical, and
verification work against one aggregate parent, so an operator can reconcile each
recorded route with the work it consumed.

## Candidate route traces explain bounded pipeline dispositions

A route returning a paper is not enough to explain whether BioMCP kept it. The
opt-in debug plan records a bounded, versioned trace of candidate-route
observations, so operators can distinguish receipt from union, deduplication,
identity verification, and pagination without exposing provider payloads. It
reports the total observations considered and the number dropped after the
bound, so a full trace is distinguishable from a truncated one. Retention puts
visible, deduplicated receipts first, then fills remaining space in observation
order. A candidate removed by `--confirmed-only` remains attributable rather
than silently disappearing from the diagnostic, and duplicate route
observations remain distinct. Every emitted result also retains a received,
unioned, deduplicated, visible route receipt in its debug trace.

## Typed PubTator linkage is bound to the returned PMID

The frozen captures use current-shaped typed `Gene` and `Variant` annotations.
A confirmation is evidence only when the returned document PMID, the requested
gene's NCBI Gene ID, and the exact returned HGVS agree through the variant's
`CorrespondingGene` facts. Missing, mismatched, or extra document identities are
incomplete provider anomalies, not contrary evidence; their presence cannot turn
a confirmed exact linkage into a contradiction. An `Association` relation without
typed linkage remains unverified. Duplicate expected-PMID documents and returned
document order cannot alter the result. The tagged linkage object keeps this proof
auditable while the legacy relation fields remain null for this path.
