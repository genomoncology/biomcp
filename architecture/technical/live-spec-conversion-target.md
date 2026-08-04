# Live-spec conversion target

Ticket 645 records the target state for the 19 remaining paths in
`scripts/run-specs.sh::SPEC_LIVE_PATHS`. It complements (and does not replace)
[the request-contract test architecture](request-contract-test-architecture.md).

## Current problem

`make verify` runs complete Markdown documents, but many of those documents
contain both deterministic assertions and source-backed assertions. A failure
therefore cannot distinguish a BioMCP regression in request construction,
decoding, orchestration, or rendering from a provider not serving a selected
record at that moment. Conversely, a live page can miss a public route that is
not exercised by its selected command, as the ERepo `--detail` defect did.

The current registry is a document-level routing mechanism for an
assertion-level decision. Removing a mixed document would also remove its
working local proof. Keeping it makes provider availability look like a product
regression.

## Target state

For every source-backed assertion, deterministic proof is layered as:

```text
CLI/entity intent -> consumed source-local RequestPlan (Tier 2)
                  -> production decoder/orchestration over a real capture (Tier 3)
                  -> fixture-backed CLI rendering, where presentation is claimed
```

Only after all applicable layers are green may the corresponding live assertion
be removed from `SPEC_LIVE_PATHS`. Existing local/static assertions stay in their
current documents; a mixed document is split only when that is necessary to
route its remaining live block accurately. `make lint`, `make test`, and
`make spec` remain green after every conversion. `make verify` remains opt-in
operator evidence, not routine product proof.

A future provider-health check, if useful, is a separately labelled,
non-gating observation. It cannot be the only proof of product behavior.

### Capture contract

A Tier 3 capture is evidence of provider shape, not test data authored to fit
our parser. Each committed capture must have a machine-readable receipt with:

- provider/source and request identity, excluding credentials and signed URLs;
- UTC capture date;
- SHA-256 of the received raw bytes;
- any minimization/redaction justification; and
- a statement that the bytes were recorded from the provider before permitted
  minimization.

Tests must feed the production decoder the recorded/minimized bytes, rather
than a response assembled from the code's expected fields. A secret-bearing
header is tested for presence or redacted mode only, never its value.
`testdata/sources/capture-receipts.json` and
`tools/check-source-capture-receipts.py` audit this admission boundary: only
`real_and_receipted` inputs are Tier 3 eligible. Synthetic and pending parser
inputs remain useful, but cannot support a conversion.

**Audit at ticket 645:** one historical code-shaped edit was confirmed:
`testdata/sources/clingen_erepo/apc-detail.json` changed the provider `@id` to
match a local-host comparison. It was re-captured in `c3092c88`; the current
confirmed byte-unfaithful count is **0**. The other 85 source files are not yet
certified: 83 have no receipt metadata, and the changed PubTator, MyVariant, and
synthetic NCBI parser inputs need explicit provenance classification. No
conversion may treat an undocumented fixture as a real capture.

## Classification

All 19 remaining live paths are **convert**. None is **keep** (provider-only smoke) or
**retire** (no remaining product contract).

| Path | Tier 2 and Tier 3 replacement before removal |
|---|---|
| `article-assets-live.md` | PubTator/OA/asset plans; dated PubTator export and PMID 20516115 asset responses. |
| `article-graph-live.md` | Semantic Scholar graph plans and redacted header-presence test; dated paper/citation/recommendation captures including empty and identifier-only cases. |
| `diagnostic.md` | GTR, WHO IVD, and OpenFDA dispatch plans; dated GTR/WHO IVD/OpenFDA captures. |
| `disease.md` | MyDisease/Monarch/NIH/SEER plans and fallback; dated Monarch plus existing-source receipts. |
| `drug.md` | MyChem/OpenFDA/EMA/WHO/ChEMBL/DDInter selection plans; dated MyChem, EMA, WHO, and local-bundle provenance. |
| `gene.md` | MyGene and optional GO/HPA/ChEMBL/NIH/GTR plans; dated captures for asserted sections. |
| `pathway.md` | KEGG/Reactome/WikiPathways plans and alias normalization; dated search/detail captures. |
| `pgx.md` | CPIC guideline/recommendation/frequency plans; dated captures for result and empty mapping. |
| `phenotype.md` | Monarch/HPO plans and typed follow-ups; dated phrase and ID captures. |
| `protein.md` | UniProt/ComplexPortal plans; dated identity, structure, and complex captures. |
| `trial.md` | CT.gov/NCI plans and cursor handling; dated cursor/contact/eligibility and NCI captures. |
| `vaers.md` | VAERS/OpenFDA plans and source eligibility; dated VAERS aggregate and unsupported/empty capture. |
| `variant-hotspots.md` | CancerHotspots plan and structure join; dated BRAF/MYD88, empty, and recurrence captures. |
| `clingen-car-live.md` | CAR transcript-HGVS plan and ordering; dated normalization capture including aliases/version. |
| `clingen-ldh-live.md` | LDH medium/direct plans, CAid selection, and bounds; dated positive, empty, malformed, and not-attempted captures. |
| `variant-myvariant-live.md` | MyVariant get/search/filter plans; dated consequence/filter payload captures. |
| `variant-articles-live.md` | strict/exact, PubTator/S2/CAR/LDH plans and budget attribution; dated positive, empty, degraded, and not-attempted panel captures. |
| `cli.md` | Retain local help/list assertions; move each external health/source block behind its owning source replacement and capture. |
| `discover.md` | consumed DiscoverRequest/OLS4 plans and typed fallbacks; dated OLS4 result, no-match, and relational captures. |

## Migration invariants

1. A live assertion is replaced only when Tier 2 covers CLI-to-API-call
   transition and Tier 3 covers production parsing of a locally captured real
   response; only current provider availability is dropped.
2. Tier 2 and Tier 3 land before the live assertion is removed.
3. Real, dated, receipt-backed captures are required; synthetic fixtures prove
   only parser self-agreement.
4. The variant-article recall thresholds and panel inputs are preserved
   verbatim, with explicit source-status and work-budget cases.
5. Semantic Scholar credential routing is proven by a local receiver observing
   redacted header presence, never by a probabilistic anonymous 429.
6. A source status may name a provider only after a call to that provider was
   recorded; not-attempted routes remain distinct from unavailable routes.
7. Each conversion updates the live registry only for its converted assertion
   and preserves mixed-document local coverage.

## Delivery slices

The work is deliberately source-shaped: capture receipts first; then ClinGen,
article, variant, ontology, clinical-entity, and remaining provider slices; and
finally the mixed CLI document. Each slice is independently shippable and uses
the standard repo gates. See `.march/blueprint.md` and the tickets created by
645 for ordering and dependencies.
