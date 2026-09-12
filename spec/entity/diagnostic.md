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

## Exact Disease Synonym Expansion

GTR disease search resolves one exact MyDisease identity, then matches the
canonical name and bounded exact synonyms without broadening to ontology
parents or fuzzy candidates. The per-row provenance is shared by JSON and the
Markdown table, and paging happens after the match rank.

```bash
../../tools/biomcp-ci search diagnostic --disease "Bachmann-Bupp syndrome" --source gtr --json \
  | jq -c '{count, accessions: [.results[].accession], result: .results[0] | {accession, disease_match}}' \
  | mustmatch like '{"count":2,"accessions":["GTR000596648.2","GTR000596649.1"],"result":{"accession":"GTR000596648.2","disease_match":{"kind":"synonym","term":"neurodevelopmental disorder with alopecia and brain abnormalities","resolved_id":"MONDO:0033642"}}}'
../../tools/biomcp-ci search diagnostic --disease "Bachmann-Bupp syndrome" --source gtr \
  | mustmatch like '|Accession|Name|Type|Manufacturer / Lab|Source|Genes|Conditions|Disease match|
Synonym: neurodevelopmental disorder with alopecia and brain abnormalities'
../../tools/biomcp-ci search diagnostic --disease "neurodevelopmental disorder with alopecia and brain abnormalities" --source gtr --limit 1 \
  | mustmatch like 'GTR000596648.2
Requested: neurodevelopmental disorder with alopecia and brain abnormalities'
../../tools/biomcp-ci search diagnostic --disease "Bachmann-Bupp syndrome" --source gtr --offset 1 --limit 1 \
  | mustmatch like 'GTR000596649.1
Showing 2 of 2 results.'
../../tools/biomcp-ci search diagnostic --disease "Bachmann-Bupp syndrome" --source gtr --manufacturer "Example Lab" --type molecular --limit 1 \
  | mustmatch like 'GTR000596648.2'
../../tools/biomcp-ci search diagnostic --disease "Bachmann-Bupp syndrome" --source gtr --manufacturer "wrong lab" --type molecular --limit 1 \
  | mustmatch like 'No diagnostic tests found.'
```

WHO-only and searches without a disease filter never contact MyDisease. An
unavailable resolver is reported safely instead of being presented as a true
zero-result search.

```bash
request_log="$BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG"
before="$(awk '/^GET \/mydisease\// { count++ } END { print count + 0 }' "$request_log")"
../../tools/biomcp-ci search diagnostic --disease HIV --source who-ivd --limit 1 >/dev/null
../../tools/biomcp-ci search diagnostic --gene BRCA1 --source gtr --limit 1 >/dev/null
after="$(awk '/^GET \/mydisease\// { count++ } END { print count + 0 }' "$request_log")"
test "$after" -eq "$before"

set +e
failure_file="$(mktemp)"
../../tools/biomcp-ci search diagnostic --disease "resolver failure syndrome" --source gtr --limit 1 >"$failure_file" 2>&1
status=$?
set -e
test "$status" -ne 0
cat "$failure_file" | mustmatch like 'Source unavailable: MyDisease.info is not available.
Review source configuration and retry.'
cat "$failure_file" | mustmatch not like 'synthetic resolver failure'
rm -f "$failure_file"
```
