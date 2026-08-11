# Diagnostic Queries

Diagnostic search has to stay source-aware: GTR and WHO IVD share one command
surface, but they do not support the same filters or detail sections. These
canaries keep that provenance, rejection guidance, and compact discovery-table
behavior visible. Routine execution uses the shared local provider fixture,
including dated OpenFDA responses captured through the production request path.

## Filter-Required Search

Diagnostic discovery is filter-driven. An empty search should fail fast with a
message that tells the user which filter families are actually supported.

```bash
../../tools/biomcp-ci search diagnostic 2>&1 | mustmatch like 'diagnostic search requires at least one of --gene, --disease, --type, or --manufacturer'
```

## Source-Aware Discovery Rows

The discovery table should keep its source column and show which source backed
each row, even when the query only matches WHO IVD results.

```bash
../../tools/biomcp-ci search diagnostic --disease HIV --limit 5 | mustmatch like '# Diagnostic tests: disease=HIV
|Accession|Name|Type|Manufacturer / Lab|Source|Genes|Conditions|
|WHO Prequalified IVD|-|HIV|'
```

## Gene-First GTR Workflows

Gene-first diagnostic search is a GTR path. WHO IVD requests should say that
plainly instead of silently pretending the gene filter worked.

```bash
../../tools/biomcp-ci search diagnostic --source who-ivd --gene BRCA1 2>&1 | mustmatch like 'WHO IVD does not support --gene
use --source gtr or omit --source for gene-first diagnostic searches'
```

## Compact Discovery Rows

Broad panel rows should stay compact in the discovery table, with overflow
markers instead of unbounded gene and condition inventories.

```bash
../../tools/biomcp-ci search diagnostic --gene BRCA1 --limit 3 | mustmatch like '# Diagnostic tests: gene=BRCA1
NCBI Genetic Testing Registry'
../../tools/biomcp-ci search diagnostic --gene BRCA1 --limit 3 | mustmatch '/\+[0-9]+ more/'
```

## Source-Aware Detail Sections

WHO detail cards should keep their supported sections visible and point users at
the next valid deepen path.

```bash
../../tools/biomcp-ci get diagnostic 'ITPW02232- TC40' conditions | mustmatch like '## Conditions
biomcp get diagnostic "ITPW02232- TC40" regulatory
WHO Prequalified IVD'
```

## Regulatory Overlay Stays Opt-In

The FDA device overlay should only appear when requested, so `all` stays
source-native instead of silently pulling in extra sections.

```bash
id="$(../../tools/biomcp-ci search diagnostic --gene BRCA1 --limit 1 | awk -F'|' '/^\|GTR/{print $2; exit}')"
../../tools/biomcp-ci get diagnostic "$id" all | mustmatch not like "## Regulatory (FDA Device)"
../../tools/biomcp-ci get diagnostic "$id" regulatory | mustmatch like "## Regulatory (FDA Device)"
```
