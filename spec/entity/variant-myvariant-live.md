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
