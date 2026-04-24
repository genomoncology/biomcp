# Pathway Queries

Pathway search and detail calls are where BioMCP has to normalize the same
biological idea across KEGG, Reactome, and WikiPathways without hiding
source-specific limits. These batch-B canaries keep alias handling, ranking,
default cards, and rejection guidance honest.

## Long-Form Alias Normalization

Long-form pathway wording should still land on the canonical pathway the user
asked for instead of drifting into nearby but unrelated pathway names.

```bash
out="$(../../tools/biomcp-ci search pathway 'mitogen activated protein kinase signaling pathway' --limit 3)"
echo "$out" | mustmatch like "# Pathways: mitogen activated protein kinase signaling pathway"
echo "$out" | mustmatch like "| KEGG | hsa04010 | MAPK signaling pathway |"
```

## Query-Required Guidance

An empty pathway search should fail with a recoverable instruction rather than
printing a blank result table.

```bash
out="$(../../tools/biomcp-ci search pathway 2>&1 || true)"
echo "$out" | mustmatch like "Query is required."
echo "$out" | mustmatch like 'biomcp search pathway -q "MAPK signaling"'
```

## Exact-Title Ranking

When the user already knows the exact pathway title, that exact-title row should
stay visible at the top of the small result set.

```bash
out="$(../../tools/biomcp-ci search pathway 'MAPK signaling pathway' --limit 3)"
first_row="$(printf '%s\n' "$out" | awk '/^\| Source \| ID \| Name \|/{getline; getline; print; exit}')"
echo "$out" | mustmatch like "| Source | ID | Name |"
echo "$first_row" | mustmatch like "| KEGG | hsa04010 | MAPK signaling pathway |"
```

## Concise KEGG Default

Default KEGG cards should stay summary-first and point users at opt-in deeper
sections instead of dumping every section by default.

```bash
out="$(../../tools/biomcp-ci get pathway hsa05200)"
echo "$out" | mustmatch like "Source: KEGG"
echo "$out" | mustmatch like "biomcp get pathway hsa05200 genes"
echo "$out" | mustmatch like "biomcp get pathway hsa05200 all"
```

## Unsupported Section Rejection

Source-aware sections should fail with specific guidance when the user asks for
a section that only exists on a different pathway source.

```bash
out="$(../../tools/biomcp-ci get pathway hsa05200 enrichment 2>&1 || true)"
echo "$out" | mustmatch like 'pathway section "enrichment" is not available for KEGG pathways'
echo "$out" | mustmatch like "Use a Reactome pathway ID such as R-HSA-5673001"
```
