# Gene Queries

Gene search is the fastest way to anchor a BioMCP session in a stable entity.
These canaries keep the focus on durable identity, deepen-path guidance, and
opt-in sections instead of volatile upstream counts or copy-edit trivia.

## Symbol-Based Search

Symbol search should still surface the canonical BRAF row in a human-scannable
table before the user pivots into deeper sections.

```bash
../../tools/biomcp-ci search gene BRAF --limit 3 | mustmatch like '# Genes: BRAF
B-Raf proto-oncogene'
```

## Search Table Contract

The search surface needs to stay readable for humans and still expose machine
follow-ups through `_meta.next_commands`.

```bash
../../tools/biomcp-ci --json search gene BRAF --limit 3 | mustmatch like '"next_commands":'
../../tools/biomcp-ci --json search gene BRAF --limit 3 | jq -e '._meta.next_commands[0] | test("^biomcp get gene .+$")' >/dev/null
../../tools/biomcp-ci --json search gene BRAF --limit 3 | jq -e '._meta.next_commands | any(. == "biomcp list gene")' >/dev/null
```

## Identity Card

The default card should keep the persistent identifier and the progressive
disclosure hints that let readers deepen into the right follow-up section.

```bash
../../tools/biomcp-ci get gene BRAF | mustmatch like 'Entrez ID: 673
biomcp get gene BRAF pathways
biomcp get gene BRAF diagnostics'
```

## Common Alias Get Resolves Canonical Gene

Clinical reports and papers often use common aliases instead of the HGNC symbol.
For an alias that maps to one canonical gene, `get gene` should return the same
stable gene card a user would get from the official symbol.

```bash
../../tools/biomcp-ci --json get gene PD-L1 | mustmatch like '"symbol": "CD274"
"entrez_id": "29126"
"PD-L1"'
```

## Diagnostics and Pathways Pivots

The base gene view advertises its diagnostic and pathway deepen paths without
requiring any optional enrichment provider.

```bash
../../tools/biomcp-ci --json get gene BRCA1 | mustmatch like '"next_commands":'
../../tools/biomcp-ci --json get gene BRCA1 | jq -e '._meta.next_commands | any(. == "biomcp get gene BRCA1 diagnostics")' >/dev/null
../../tools/biomcp-ci --json get gene BRCA1 | jq -e '._meta.next_commands | any(. == "biomcp get gene BRCA1 pathways")' >/dev/null
```

## Observed MyGene Requests

The local fixture records requests emitted by the production client, including
the bounded search and exact-symbol identity plans.

```bash
grep -F 'GET /mygene/v3/query?q=%28symbol%3ABRAF+OR+alias%3ABRAF%29' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like '&size=3&from=0'
grep -F 'GET /mygene/v3/query?q=symbol%3A%22BRCA1%22' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like '&size=1'
```

## Typed optional-section outcomes

Requested sections keep a bounded state even when providers return no rows or
are temporarily unavailable. Provenance carries the same state rather than
inferring success from an empty collection.

```bash
../../tools/biomcp-ci --json get gene BRAF go interactions \
  | jq '. as $root | ["go", "interactions"] | all(.[]; . as $key | $root.section_outcomes[$key] as $outcome | ($outcome.outcome | IN("data", "empty", "unavailable")) and ($root._meta.section_sources | any(.key == $key and .outcome == $outcome.outcome and .sources == $outcome.sources)) and ($root._meta.section_sources | all(.key != $key or (.outcome == $outcome.outcome and .sources == $outcome.sources))))' \
  | mustmatch 'true'
grep -F 'GET /quickgo/QuickGO/services/annotation/search?geneProductId=P15056&limit=20' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like 'P15056'
grep -F 'GET /string/api/json/network?identifiers=BRAF&species=9606&limit=15' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like 'species=9606'
```

## All-Section Warm Budget

Quarantined from routine executable specs by ticket 372 because this timing-only
canary failed twice during routine `make spec-pr` at 45599ms and 43332ms against
a 12000ms ceiling. Ticket 371's request-contract strategy keeps performance
canaries out of the default gate; restore this only as a deterministic
benchmark/ratchet or explicit performance lane.

## Tissue-Expression Context

Human Protein Atlas data belongs in an opt-in deepen path and retains its
source reliability and subcellular context.

```bash
../../tools/biomcp-ci get gene BRAF hpa | mustmatch like '## Human Protein Atlas
| Adipose tissue | Low |
Reliability: Supported
Subcellular main locations: cytosol, vesicles'
grep -F 'GET /hpa/ENSG00000157764.xml' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like 'ENSG00000157764.xml'
```

## Druggability & Targets

Targetability context stays separate from the default card while combining
Open Targets tractability and DGIdb interaction evidence.

```bash
../../tools/biomcp-ci get gene EGFR druggability | mustmatch like '## Druggability
OpenTargets tractability
| antibody | yes | Approved Drug'
grep -F 'POST /dgidb/api/graphql' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like '"gene":"EGFR"'
grep -F 'POST /opentargets/api/v4/graphql' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like '"ensemblId":"ENSG00000146648"'
```

## Funding

Funding remains opt-in and retains its source-attributed bounded table.

```bash
../../tools/biomcp-ci get gene ERBB2 funding | mustmatch like '## Funding (NIH Reporter)
| Project | PI | Organization | FY | Amount |
Showing top 8 unique grants from 187 matching NIH project-year records'
grep -F 'POST /nih/v2/projects/search' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like '"search_text":"\"ERBB2\""'
```

## Diagnostic Local Data

The diagnostic deepen path consumes the bounded local GTR bundle rather than
downloading provider data during the routine gate.

```bash
../../tools/biomcp-ci get gene BRCA1 diagnostics | mustmatch like '## Diagnostics
GTR000000001.1
NCBI Genetic Testing Registry'
```
