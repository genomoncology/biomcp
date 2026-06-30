# Disease Queries

Disease workflows are where BioMCP has to normalize human language onto stable
ontology IDs while still keeping treatment and diagnostic pivots close at hand.
These batch-A canaries focus on MONDO grounding, synonym rescue, section gating,
and executable follow-up guidance.

## Disease Request Planning Happens Before MyDisease Calls

Disease search first records normalized command intent in request seams before
MyDisease or discover fallback clients execute. The search seam carries query,
filters, pagination, resolver queries, fetch sizing, and DOID preference; the
fallback seam separately records MESH skip, alias-fallback discover mode, and
crosswalk resolution intent.

## Deterministic Renderer Envelope Contracts

Ticket 377 moves routine disease renderer/envelope proof into fixture-result
contracts. The deterministic tests should cover disease JSON `_meta.next_commands`,
source provenance, markdown table/card anchors, and follow-up guidance without
making live MyDisease, OLS4, Open Targets, GTR, or trial calls.

```bash
cargo test --lib ticket_377_disease_renderer_envelope_contracts -- --nocapture \
  | mustmatch like 'ticket_377_disease_renderer_envelope_contracts'
```

## Disease Normalization & Search

Direct disease search should still surface the canonical melanoma row with its
MONDO identifier visible in the result table.

## Synonym Rescue

Ticket 371 identified this live OLS4/MyDisease path as a request-contract risk;
routine coverage for the Arnold/Chiari synonym rescue path is now restored
through Rust fixture/request-command/request-plan tests. Disease search and
fallback request seams preserve fallback intent before execution, fallback
ranking is fixture-backed, OLS4 search construction is asserted by
`OlsSearchRequestPlan`, and MyDisease MESH crosswalk construction is asserted by
`MyDiseaseXrefLookupRequestPlan`. Any live OLS4/MyDisease upstream probe belongs
in a release/live-smoke lane, not routine `make spec-pr`.

## Canonical Disease Card

The default card should expose the persistent ID, top cross-entity summaries,
and the executable next steps for trials, articles, diagnostics, and drugs.

```bash
../../tools/biomcp-ci get disease melanoma | mustmatch like 'ID: MONDO:0005105
Recruiting Trials (ClinicalTrials.gov):
biomcp search trial -c "melanoma"
biomcp search drug --indication "melanoma"'
```

## Genes & Diagnostics

`genes` and `diagnostics` stay opt-in sections, but when requested they should
render as explicit tables and admit that the diagnostic list is truncated.

## Clinical Features

Clinical features are a separate opt-in MedlinePlus section for reviewed
configured diseases. Uterine leiomyoma should render source-native symptom rows
with reviewed HPO mappings instead of falling back to the broader phenotype
section or a blank table.

```bash
../../tools/biomcp-ci get disease "uterine leiomyoma" clinical_features | mustmatch like '## Clinical Features (MedlinePlus)
| Rank | Feature | HPO | Confidence | Evidence | Source |
heavy menstrual bleeding
HP:0000132 (Menorrhagia)
[MedlinePlus](https://medlineplus.gov/uterinefibroids.html)'
```

## NIH Funding Context

Funding belongs in its own section. The card should keep that view truthful and
bounded instead of implying the first page is the whole research landscape.

```bash
../../tools/biomcp-ci get disease 'Marfan syndrome' funding | mustmatch like '## Funding (NIH Reporter)
| Project | PI | Organization | FY | Amount |'
```

## JSON Pivots

The JSON card should keep the same executable disease follow-ups that the
markdown card teaches to humans.

## Disease Survival Commands Exit After Rendering (Live SEER)

Relocated from the routine `cli-contract-ratchet.md` into the live lane: the
survival card is sourced live from SEER Explorer, so this bounded-exit check must
run in the live `verify` lane, not the deterministic routine gate. Disease
survival cards are useful to agents only when the process exits after printing
them; both command forms share the same disease-survival execution path, so each
is bounded by `timeout` and asserts survival-card landmarks rather than exact
percentages.

```bash
set -o pipefail
timeout 20s ../../tools/biomcp-ci get disease --name "chronic myeloid leukemia" survival | mustmatch like '## Survival (SEER Explorer)
Source: Chronic Myeloid Leukemia (CML)'
```

```bash
set -o pipefail
timeout 20s ../../tools/biomcp-ci get disease "chronic myeloid leukemia" survival | mustmatch like '## Survival (SEER Explorer)
Source: Chronic Myeloid Leukemia (CML)'
```
