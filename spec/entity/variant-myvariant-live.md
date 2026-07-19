# Live MyVariant Variant Contracts

These operator-run canaries exercise BioMCP against the real MyVariant service.
They protect source paths that can fail silently when MyVariant's indexed schema
changes.

## BayesDel prediction flavors

BayesDel's add-AF and no-AF scores have different meanings and thresholds. A
variant carrying both upstream values must expose two populated, unambiguous
prediction entries rather than silently dropping or combining them.

```bash
biomcp --json --no-cache get variant 'chr7:g.140453136A>T' predictions \
  | jq -r '
      .expanded_predictions[]?
      | select(
          (.tool == "BayesDel add-AF" or .tool == "BayesDel no-AF")
          and (.score | type == "number")
          and (.prediction | type == "string" and length > 0)
        )
      | .tool
    ' \
  | mustmatch like 'BayesDel add-AF
BayesDel no-AF'
```

## GERP-filtered variant search

A minimum GERP score should return indexed variants that meet the conservation
filter. This positive result check catches a stale field path that turns a valid
filter into a successful but empty response.

```bash
biomcp --json --no-cache search variant --gerp-min 4 --limit 5 \
  | jq '(.results | length > 0) and (.results | all((.gerp | type == "number") and (.gerp >= 4)))' \
  | mustmatch 'true'
```

## Consequence-filtered variant search

Sequence Ontology consequence names are translated to MyVariant's indexed
annotations. The common BRAF V600E missense path must return a variant instead
of a successful empty result.

```bash
biomcp --json --no-cache search variant -g BRAF --hgvsp V600E \
  --consequence missense_variant --limit 5 \
  | jq '(.results | length > 0)' \
  | mustmatch 'true'
```

## Supported consequence matrix

Less common coding and splice consequences use the same public vocabulary. Each
supported term must select real records rather than disappearing or returning a
confident empty result.

| str:term | str:label |
|---|---|
| synonymous_variant | synonymous |
| frameshift_variant | frameshift |
| splice_donor_variant | splice donor |
| inframe_deletion | in-frame deletion |

```bash run id=live-consequence each_row="Supported consequence matrix"
biomcp --json --no-cache search variant --consequence {{term}} --limit 3 | jq '(.results | length > 0)'
```

```text expect=live-consequence each_row="Supported consequence matrix"
true
```

## ClinVar review-status filtering

ClinVar star aliases map to the provider's review phrases. A two-star BRCA1
search returns only rows with at least that exposed review rating, and the
multi-filter command printed by CLI help remains a working example.

```bash
biomcp --json --no-cache search variant -g BRCA1 --review-status 2 --limit 5 \
  | jq '(.results | length > 0) and (.results | all(.clinvar_stars >= 2))' \
  | mustmatch 'true'
```

```bash
biomcp --json --no-cache search variant -g BRCA1 --review-status 2 \
  --revel-min 0.7 --consequence missense_variant --limit 5 \
  | jq '(.results | length > 0)' \
  | mustmatch 'true'
```

## Field presence and absence filtering

Field aliases represent fields BioMCP actually returns. Requiring REVEL yields
scored rows, while requiring it to be missing yields rows without a score.

```bash
biomcp --json --no-cache search variant -g BRAF --has revel --limit 5 \
  | jq '(.results | length > 0) and (.results | all(.revel | type == "number"))' \
  | mustmatch 'true'
```

```bash
biomcp --json --no-cache search variant -g BRAF --missing revel --limit 5 \
  | jq '(.results | length > 0) and (.results | all(.revel == null))' \
  | mustmatch 'true'
```
