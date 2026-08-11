# Live Disease Queries

These operator-run checks retain disease behaviors that have not yet moved to a local captured-response contract. They remain separate from the routine ontology contracts in [Disease Queries](disease.md).

## Disease Normalization & Search

Direct disease search should still surface the canonical melanoma row with its
MONDO identifier visible in the result table. Supported inheritance and onset
filters remain accepted when narrowing that search; a live provider may
legitimately return an empty filtered page.

```bash
set -o pipefail
../../tools/biomcp-ci --json search disease -q melanoma --inheritance "autosomal dominant" --limit 1 --no-fallback \
  | jq 'has("results") and (.results | type == "array")' \
  | mustmatch 'true'
```

```bash
set -o pipefail
../../tools/biomcp-ci --json search disease -q melanoma --onset adult --limit 1 --no-fallback \
  | jq 'has("results") and (.results | type == "array")' \
  | mustmatch 'true'
```

## Canonical Disease Card

The default card should expose the persistent ID, top cross-entity summaries,
and the executable next steps for trials, articles, diagnostics, and drugs.

```bash
../../tools/biomcp-ci get disease melanoma | mustmatch like 'ID: MONDO:0005105
Recruiting Trials (ClinicalTrials.gov):
biomcp search trial -c "melanoma"
biomcp search drug --indication "melanoma"'
```
