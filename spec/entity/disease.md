# Disease Queries

Disease workflows normalize human language onto stable ontology IDs while keeping treatment and diagnostic pivots close at hand. These captured contracts replay recorded provider responses through the shipped CLI rather than treating current upstream availability as product behavior.

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


## Captured Ontology Clinical Features

The routine ontology fixture grounds chronic myeloid leukemia and serves the recorded Monarch association response locally. This proves that the public card carries a decoded HPO feature and its provenance, rather than merely rendering an empty clinical-features shell.

```bash
../../tools/biomcp-ci get disease "chronic myeloid leukemia" clinical_features \
  | awk '/^## Clinical Features/{inside=1; next} inside && /^## /{exit} inside' \
  | mustmatch like '| HP:0005547 | Myeloproliferative disorder | has phenotype | Obligate | - | - | - | [infores:orphanet](https://www.orpha.net/en/disease/detail/521) |'
```

## Captured NIH Funding Context

The local provider fixture replays the receipted NIH Reporter search for Marfan syndrome. A funding card must retain the funding table and a non-empty grants collection, so callers do not mistake a dropped response for an empty research landscape.

```bash
../../tools/biomcp-ci --json get disease "Marfan syndrome" funding | jq '.funding.grants | length > 0' | mustmatch 'true'
```

## Captured Survival Card

The ontology fixture also joins the captured CML identity to the recorded SEER catalog and survival payload. This disease-page contract keeps the named lookup form on the public rendering path and catches a regression that loses the survival card or fails to exit.

<!-- mustmatch-lint: skip -->

```bash run id=captured-disease-survival exit=0 timeout=25
timeout 20s ../../tools/biomcp-ci get disease --name "chronic myeloid leukemia" survival
```

```text expect=captured-disease-survival contains
## Survival (SEER Explorer)
Source: Chronic Myeloid Leukemia (CML)
| Sex | Latest observed year | 5-year relative survival | 95% CI | Cases | Latest modeled |
Both Sexes
```

## Observed Disease Provider Requests

The fixture records the requests emitted by production clients for identity,
phenotype, funding, and survival data.

```bash
grep -F 'GET /mydisease/query?q=%28disease_ontology.name%3Achronic+myeloid+leukemia' "$BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG" | mustmatch like 'size=15&from=0'
grep -F 'GET /monarch/v3/api/association?subject=MONDO%3A0011996' "$BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG" | grep -F 'limit=80' | mustmatch like 'object_category=biolink%3APhenotypicFeature'
grep -F 'POST /nih/v2/projects/search' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like '"search_text":"\"Marfan syndrome\""'
grep -F 'GET /seer/render_region_5.php' "$BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG" | mustmatch like 'site=97&data_type=4'
```

## Synonym Rescue

Ticket 371 identified this live OLS4/MyDisease path as a request-contract risk;
routine coverage for the Arnold/Chiari synonym rescue path is now restored
through Rust fixture/request-command/request-plan tests. Disease search and
fallback request seams preserve fallback intent before execution, fallback
ranking is fixture-backed, OLS4 search construction is asserted by
`OlsSearchRequestPlan`, and MyDisease MESH crosswalk construction is asserted by
`MyDiseaseXrefLookupRequestPlan`. Any live OLS4/MyDisease upstream probe belongs
in a release/live-smoke lane, not routine `make spec-pr`.

## Common Abbreviation Search

Ticket 1295. MyDisease indexes `mondo.synonym` and `disease_ontology.synonyms`
as objects scoped by synonym type; only the `exact` leaf fields hold tokens, so
the search query targets those leaf fields for exact synonym and abbreviation
retrieval, and BioMCP adds no abbreviations of its own. The abbreviation source
is the provider's own exact synonym lists, strict to the exact scope. The
abbreviation table below records, from responses captured on 2026-10-05, which
of the ticket's abbreviations the source holds. `CAD` is ambiguous in the
source's own lists (coronary artery disease, cold agglutinin disease, and
alveolar capillary dysplasia each hold it), so search resolves it to every
exact holder. `HGSC` is held by no exact synonym list in the source, so it
returns no rows and stays a deferred gap; the parent is reachable by its full
name.

| Input | Expected first identifier |
| --- | --- |
| NSCLC | MONDO:0005233 |
| DLBCL | MONDO:0018905 |
| CRC | MONDO:0024331 |
| AML | MONDO:0018874 |
| CAD | all three exact holders (see below) |
| HGSC | none (no source synonym) |
| non-small cell lung carcinoma | MONDO:0005233 |
| diffuse large B-cell lymphoma | MONDO:0018905 |
| colorectal carcinoma | MONDO:0024331 |
| acute myeloid leukemia | MONDO:0018874 |
| ovarian serous carcinoma | MONDO:0005211 |

```bash
set -o pipefail
first_id() {
  ../../tools/biomcp-ci --json search disease -q "$1" --no-fallback --limit 3 \
    | jq -r '.results[0].id // "none"'
}
{
  first_id NSCLC
  first_id DLBCL
  first_id CRC
  first_id AML
  first_id HGSC
  first_id "non-small cell lung carcinoma"
  first_id "diffuse large B-cell lymphoma"
  first_id "colorectal carcinoma"
  first_id "acute myeloid leukemia"
  first_id "ovarian serous carcinoma"
} | mustmatch 'MONDO:0005233
MONDO:0018905
MONDO:0024331
MONDO:0018874
none
MONDO:0005233
MONDO:0018905
MONDO:0024331
MONDO:0018874
MONDO:0005211'
```

```bash
set -o pipefail
../../tools/biomcp-ci --json search disease -q CAD --no-fallback --limit 3 \
  | jq '([.results[].id] | sort) == ["MONDO:0005010", "MONDO:0018922", "MONDO:0100077"]' \
  | mustmatch 'true'
```


## Genes & Diagnostics

`genes` and `diagnostics` stay opt-in sections, but when requested they should
render as explicit tables and admit that the diagnostic list is truncated.


## JSON Pivots

The JSON card should keep the same executable disease follow-ups that the
markdown card teaches to humans.
