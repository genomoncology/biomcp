# Pathway Queries

Pathway search should normalize a small set of confirmed alias phrases before querying the current pathway sources. These checks focus on the long-form MAPK regression without relying on unstable upstream totals.

| Section | Command focus | Why it matters |
|---|---|---|
| Long-form alias search | `search pathway 'mitogen activated protein kinase'` | Confirms alias normalization to MAPK |
| Unsupported KEGG `events` | `get pathway hsa05200 events` | Confirms explicit unsupported section requests fail hard |
| Unsupported KEGG `enrichment` | `get pathway hsa05200 enrichment` | Confirms KEGG enrichment no longer degrades to blank success |

## Long-Form MAPK Alias

The confirmed long-form MAPK phrase should return MAPK-named pathways instead of unrelated protein kinase results. This guards the narrow alias-normalization fix introduced for pathway search.

```bash
out="$("$(git rev-parse --show-toplevel)/target/release/biomcp" search pathway "mitogen activated protein kinase" --limit 5)"
echo "$out" | mustmatch like "# Pathways: mitogen activated protein kinase"
echo "$out" | mustmatch like "| Source | ID | Name |"
echo "$out" | mustmatch like "MAPK"
```

## Unsupported KEGG Events Section

Explicit unsupported KEGG section requests must fail non-zero with a truthful error
message instead of returning a blank or near-blank success page.

```bash
unset status
out="$(biomcp get pathway hsa05200 events 2>&1)" || status=$?
test "${status:-0}" -ne 0
echo "$out" | mustmatch like 'Invalid argument: pathway section "events"'
echo "$out" | mustmatch like "KEGG"
echo "$out" | mustmatch like "Reactome"
```

## Unsupported KEGG Enrichment Section

KEGG pathway enrichment is unsupported for this contract and must fail before render
time, including in the standard CLI surface.

```bash
unset status
out="$(biomcp get pathway hsa05200 enrichment 2>&1)" || status=$?
test "${status:-0}" -ne 0
echo "$out" | mustmatch like 'Invalid argument: pathway section "enrichment"'
echo "$out" | mustmatch like "KEGG"
echo "$out" | mustmatch like "Reactome"
```
