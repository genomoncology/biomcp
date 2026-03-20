# Pathway Queries

Pathway search should normalize a small set of confirmed alias phrases before querying the current pathway sources. These checks focus on the long-form MAPK regression without relying on unstable upstream totals.

| Section | Command focus | Why it matters |
|---|---|---|
| Long-form alias search | `search pathway 'mitogen activated protein kinase'` | Confirms alias normalization to MAPK |
| Default KEGG card stays concise | `get pathway hsa05200` | Confirms KEGG base cards keep genes behind explicit section requests |
| Explicit KEGG genes section still renders | `get pathway hsa05200 genes` | Confirms the concise default does not break explicit deep-section requests |
| Exact title match ranks first across sources | `search pathway 'Pathways in cancer'` | Confirms exact matches float ahead of weaker cross-source hits |
| Query required unless `--top-level` | `search pathway` / `search pathway --top-level` | Confirms the queryless contract and recovery guidance |
| Parser usage errors exit 2 | `search pathway --badflag` | Confirms clap failures keep a separate exit-code category |
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

## Default KEGG Card Stays Concise

KEGG base cards should remain summary-first unless the caller explicitly asks for
the `genes` section. This regression checks both markdown output and JSON shape.

```bash
out="$(biomcp get pathway hsa05200)"
echo "$out" | mustmatch like "# Pathways in cancer"
echo "$out" | mustmatch not like "## Genes"
echo "$out" | mustmatch not like "BRAF"

json_out="$(biomcp --json get pathway hsa05200)"
echo "$json_out" | mustmatch like '"genes": []'
```

## Explicit KEGG Genes Section Still Renders

Requesting `genes` explicitly should still return the KEGG gene section after the
default-card fix.

```bash
out="$(biomcp get pathway hsa05200 genes)"
echo "$out" | mustmatch like "## Genes"
echo "$out" | mustmatch like "BRAF"
```

## Exact Title Match Ranks First Across Sources

When a query exactly matches a pathway title, that exact row should surface first
even when other sources return weaker matches nearby.

```bash
out="$("$(git rev-parse --show-toplevel)/target/release/biomcp" search pathway "Pathways in cancer" --limit 3)"
echo "$out" | mustmatch like "| Source | ID | Name |"
printf '%s\n' "$out" | grep "^| " | tail -n +2 | head -1 | mustmatch like "| KEGG | hsa05200 | Pathways in cancer |"
```

## Search Query Is Required Unless `--top-level`

Normal pathway search requires a query. `--top-level` is the only queryless
search mode, and the remediation example must be shell-safe for multi-word
queries.

```bash
unset status
out="$(biomcp search pathway 2>&1)" || status=$?
test "${status:-0}" -eq 1
echo "$out" | mustmatch like 'Invalid argument: Query is required.'
echo "$out" | mustmatch like 'biomcp search pathway -q "MAPK signaling"'
```

```bash
unset status
out="$(biomcp search pathway --top-level 2>&1)" || status=$?
test "${status:-0}" -eq 0
echo "$out" | mustmatch like "# Pathways"
```

## Parser Usage Errors Exit 2

Clap parser failures should stay distinct from BioMCP runtime argument
validation failures.

```bash
unset status
out="$(biomcp search pathway --badflag 2>&1)" || status=$?
test "${status:-0}" -eq 2
echo "$out" | mustmatch like "unexpected argument '--badflag'"
echo "$out" | mustmatch like "Usage: biomcp search pathway"
```

## Unsupported KEGG Events Section

Explicit unsupported KEGG section requests must fail non-zero with a truthful error
message instead of returning a blank or near-blank success page.

```bash
unset status
out="$(biomcp get pathway hsa05200 events 2>&1)" || status=$?
test "${status:-0}" -eq 1
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
test "${status:-0}" -eq 1
echo "$out" | mustmatch like 'Invalid argument: pathway section "enrichment"'
echo "$out" | mustmatch like "KEGG"
echo "$out" | mustmatch like "Reactome"
```

## WikiPathways Search Presence

Normal pathway search should include WikiPathways results alongside Reactome and KEGG.

```bash
out="$("$(git rev-parse --show-toplevel)/target/release/biomcp" search pathway "apoptosis" --limit 10)"
echo "$out" | mustmatch like "| Source | ID | Name |"
echo "$out" | mustmatch like "WikiPathways"
echo "$out" | mustmatch like "WP"
```

## WikiPathways Pathway Detail

`get pathway WP254` should return a pathway card with WikiPathways source attribution.

```bash
out="$(biomcp get pathway WP254)"
echo "$out" | mustmatch like "Source: WikiPathways"
echo "$out" | mustmatch like "WP254"
echo "$out" | mustmatch like "Homo sapiens"
```

## WikiPathways Genes Section

`get pathway WP254 genes` should resolve xref Entrez IDs to HGNC symbols via MyGene.

```bash
out="$(biomcp get pathway WP254 genes)"
echo "$out" | mustmatch like "## Genes"
echo "$out" | mustmatch like "WikiPathways"
```

## Unsupported WikiPathways Events Section

WikiPathways does not support the `events` section; the command must fail non-zero
with a helpful error pointing to Reactome.

```bash
unset status
out="$(biomcp get pathway WP254 events 2>&1)" || status=$?
test "${status:-0}" -eq 1
echo "$out" | mustmatch like 'Invalid argument: pathway section "events"'
echo "$out" | mustmatch like "WikiPathways"
echo "$out" | mustmatch like "Reactome"
```

## Unsupported WikiPathways Enrichment Section

WikiPathways does not support the `enrichment` section; the command must fail non-zero
with a helpful error pointing to Reactome.

```bash
unset status
out="$(biomcp get pathway WP254 enrichment 2>&1)" || status=$?
test "${status:-0}" -eq 1
echo "$out" | mustmatch like 'Invalid argument: pathway section "enrichment"'
echo "$out" | mustmatch like "WikiPathways"
echo "$out" | mustmatch like "Reactome"
```
