# Pathway Queries

Pathway search and detail calls are where BioMCP has to normalize the same
biological idea across KEGG, Reactome, and WikiPathways without hiding
source-specific limits. These batch-B canaries keep alias handling, ranking,
default cards, and rejection guidance honest.

## Long-Form Alias Normalization

Long-form pathway wording should still keep the query echo and return pathway
rows with MAPK context rather than drifting into an empty or unrelated surface.

```bash
../../tools/biomcp-ci search pathway 'mitogen activated protein kinase signaling pathway' --limit 3 | mustmatch like '# Pathways: mitogen activated protein kinase signaling pathway
| Source | ID | Name |
MAPK'
```

## Query-Required Guidance

An empty pathway search should fail with a recoverable instruction rather than
printing a blank result table.

```bash
../../tools/biomcp-ci search pathway 2>&1 | mustmatch like 'Query is required.
biomcp search pathway -q "MAPK signaling"'
```

## Exact-Title Ranking

When the user already knows the pathway title, the small result set should keep
source-identified pathway rows visible instead of returning an empty card.

```bash
../../tools/biomcp-ci search pathway 'MAPK signaling pathway' --limit 3 | mustmatch like '| Source | ID | Name |'
../../tools/biomcp-ci search pathway 'MAPK signaling pathway' --limit 3 | mustmatch '/\| (KEGG|Reactome|WikiPathways) \| [^|]+ \| [^|]*MAPK[^|]* \|/'
```

## Exact KEGG Title Participation

The same exact title is also the stable KEGG pathway label for `hsa04010`.
When the live KEGG source participates, search should expose that exact-title
row instead of silently returning only non-KEGG matches.

```bash
../../tools/biomcp-ci search pathway 'MAPK signaling pathway' --limit 5 | mustmatch like '| KEGG | hsa04010 | MAPK signaling pathway - Homo sapiens (human) |'
```

## Concise KEGG Default

Default KEGG cards should stay summary-first and point users at opt-in deeper
sections instead of dumping every section by default.

```bash
../../tools/biomcp-ci get pathway hsa05200 | mustmatch like 'Source: KEGG
biomcp get pathway hsa05200 genes
biomcp get pathway hsa05200 all'
```

## Typed optional-section outcomes

Supported Reactome sections retain a bounded result state in both the entity and
its provenance, instead of making empty vectors stand for every outcome.

```bash
../../tools/biomcp-ci --json get pathway R-HSA-5673001 genes events \
  | jq '. as $root | ["genes", "events"] | all(.[]; . as $key | $root.section_outcomes[$key] as $outcome | ($outcome.outcome | IN("data", "empty", "unavailable")) and ($root._meta.section_sources | any(.key == $key and .outcome == $outcome.outcome and .sources == $outcome.sources)) and ($root._meta.section_sources | all(.key != $key or (.outcome == $outcome.outcome and .sources == $outcome.sources))))' \
  | mustmatch 'true'
```

## Unsupported Section Rejection

Source-aware sections should fail with specific guidance when the user asks for
a section that only exists on a different pathway source.

```bash
../../tools/biomcp-ci get pathway hsa05200 enrichment 2>&1 | mustmatch like 'pathway section "enrichment" is not available for KEGG pathways
Use a Reactome pathway ID such as R-HSA-5673001'
```
