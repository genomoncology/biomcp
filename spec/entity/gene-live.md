# Gene Optional Enrichment Live Queries

These optional enrichment checks remain live until ticket 0907 adds receipted
QuickGO, HPA, target, funding, and diagnostic provider contracts.

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

## Funding

Funding remains opt-in and must retain its source-attributed table.

```bash
../../tools/biomcp-ci get gene ERBB2 funding | mustmatch like '## Funding (NIH Reporter)
| Project | PI | Organization | FY | Amount |'
```
