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
