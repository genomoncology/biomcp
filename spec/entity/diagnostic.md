# Diagnostic Queries

Diagnostic search has to stay source-aware: GTR and WHO IVD share one command
surface, but they do not support the same filters or detail sections. These
canaries keep that provenance, rejection guidance, and compact discovery-table
behavior visible.

## Filter-Required Search

Diagnostic discovery is filter-driven. An empty search should fail fast with a
message that tells the user which filter families are actually supported.

```bash
out="$(../../tools/biomcp-ci search diagnostic 2>&1 || true)"
echo "$out" | mustmatch like "diagnostic search requires at least one of --gene, --disease, --type, or --manufacturer"
```

## Source-Aware Discovery Rows

The discovery table should keep its source column and show which source backed
each row, even when the query only matches WHO IVD results.

```bash
out="$(../../tools/biomcp-ci search diagnostic --disease HIV --limit 5)"
echo "$out" | mustmatch like "# Diagnostic tests: disease=HIV"
echo "$out" | mustmatch like "|Accession|Name|Type|Manufacturer / Lab|Source|Genes|Conditions|"
echo "$out" | mustmatch like "|WHO Prequalified IVD|-|HIV|"
```

## Gene-First GTR Workflows

Gene-first diagnostic search is a GTR path. WHO IVD requests should say that
plainly instead of silently pretending the gene filter worked.

```bash
out="$(../../tools/biomcp-ci search diagnostic --source who-ivd --gene BRCA1 2>&1 || true)"
echo "$out" | mustmatch like "WHO IVD does not support --gene"
echo "$out" | mustmatch like "use --source gtr or omit --source for gene-first diagnostic searches"
```

## Compact Discovery Rows

Broad panel rows should stay compact in the discovery table, with overflow
markers instead of unbounded gene and condition inventories.

```bash
out="$(../../tools/biomcp-ci search diagnostic --gene BRCA1 --limit 3)"
echo "$out" | mustmatch like "# Diagnostic tests: gene=BRCA1"
echo "$out" | mustmatch '/\+[0-9]+ more/'
echo "$out" | mustmatch like "NCBI Genetic Testing Registry"
```

## Source-Aware Detail Sections

WHO detail cards should keep their supported sections visible and point users at
the next valid deepen path.

```bash
out="$(../../tools/biomcp-ci get diagnostic 'ITPW02232- TC40' conditions)"
echo "$out" | mustmatch like "## Conditions"
echo "$out" | mustmatch like 'biomcp get diagnostic "ITPW02232- TC40" regulatory'
echo "$out" | mustmatch like "WHO Prequalified IVD"
```

## Regulatory Overlay Stays Opt-In

The FDA device overlay should only appear when requested, so `all` stays
source-native instead of silently pulling in extra sections.

```bash
id="$(../../tools/biomcp-ci search diagnostic --gene BRCA1 --limit 1 | awk -F'|' '/^\|GTR/{print $2; exit}')"
all_out="$(../../tools/biomcp-ci get diagnostic "$id" all)"
reg_out="$(../../tools/biomcp-ci get diagnostic "$id" regulatory)"
echo "$all_out" | mustmatch not like "## Regulatory (FDA Device)"
echo "$reg_out" | mustmatch like "## Regulatory (FDA Device)"
```
