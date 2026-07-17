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

## Typed optional-section outcomes

Requested sections keep a bounded state even when live providers return no rows
or are temporarily unavailable. Provenance carries the same state rather than
inferring success from an empty collection.

```bash
../../tools/biomcp-ci --json get gene BRAF go interactions \
  | jq '. as $root | ["go", "interactions"] | all(.[]; . as $key | $root.section_outcomes[$key] as $outcome | ($outcome.outcome | IN("data", "empty", "unavailable")) and ($root._meta.section_sources | any(.key == $key and .outcome == $outcome.outcome and .sources == $outcome.sources)) and ($root._meta.section_sources | all(.key != $key or (.outcome == $outcome.outcome and .sources == $outcome.sources))))' \
  | mustmatch 'true'
```

## All-Section Warm Budget

Quarantined from routine executable specs by ticket 372 because this timing-only
canary failed twice during routine `make spec-pr` at 45599ms and 43332ms against
a 12000ms ceiling. Ticket 371's request-contract strategy keeps live-source and
performance canaries out of the default gate until they have deterministic
coverage; restore this behavior as a benchmark/ratchet or explicit performance
lane, not as a routine live-heavy spec blocker.

## Tissue-Expression Context

Human Protein Atlas data belongs in an opt-in deepen path. When live HPA data is
missing, BioMCP should stay truthful rather than fabricating tissue rows.

```bash
../../tools/biomcp-ci get gene BRAF hpa \
  | mustmatch '/## Human Protein Atlas[\s\S]*(No Human Protein Atlas records returned|\| Tissue \| Level \|[\s\S]*Reliability:[\s\S]*Subcellular)/'
```

## Druggability & Targets

Targetability context should stay separate from the default card while still
showing the combined OpenTargets and DGIdb story for actionable genes.

```bash
../../tools/biomcp-ci get gene EGFR druggability | mustmatch like '## Druggability
OpenTargets tractability
| antibody | yes | Approved Drug'
```

## Funding & Diagnostics Cross-Pivot

Funding remains opt-in, but the base gene view still needs to advertise the
diagnostics deepen path so operators can move from one card into the next.

```bash
../../tools/biomcp-ci get gene ERBB2 funding | mustmatch like '## Funding (NIH Reporter)
| Project | PI | Organization | FY | Amount |'
```

```bash
../../tools/biomcp-ci --json get gene BRCA1 | mustmatch like '"next_commands":'
../../tools/biomcp-ci --json get gene BRCA1 | jq -e '._meta.next_commands | any(. == "biomcp get gene BRCA1 diagnostics")' >/dev/null
../../tools/biomcp-ci --json get gene BRCA1 | jq -e '._meta.next_commands | any(. == "biomcp get gene BRCA1 pathways")' >/dev/null
```
