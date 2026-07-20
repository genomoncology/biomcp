# How to: find articles

This guide shows practical literature-search patterns.

## Translate a question into filters

When the gene, disease, or drug is already known, put that anchor in a typed
flag and keep the mechanism, phenotype, dataset, or outcome in `-k`.

Known anchor plus concept:

```bash
biomcp search article -g TP53 -k "apoptosis gene regulation" --limit 5
```

Unknown entity, keyword first:

```bash
biomcp search article -k '"cafe-au-lait spots" neurofibromas disease' --type review --limit 5
```

Do not guess `-g`, `-d`, or `--drug` when the question is trying to identify
the entity itself. Keep the first search keyword-only, or start with
`biomcp discover "<question>"` if you want a typed follow-up command first.
Question-format terms can stay in the article filters: PubMed ESearch cleans
bounded filler words from unfielded gene, disease, drug, and keyword terms
provider-locally, while query echoes and non-PubMed sources keep the original
wording.

If the whole keyword exactly matches a gene, drug, or disease vocabulary label
or alias, keyword-only article search may return a typed `get` suggestion in
`See also`, `_meta.next_commands`, and JSON `_meta.suggestions[]`. Treat that
as a structured follow-up option, but do not expect direct entity suggestions
for multi-concept phrases such as `BRAF V600E` or `lung cancer immunotherapy`.

Dataset or method question:

```bash
biomcp search article -k "TCGA mutation analysis dataset" --type review --limit 5
```

Refine with typed flags before paginating:

```bash
biomcp search article --drug amiodarone -k "photosensitivity mechanism" --limit 5
```

If the first page reveals the gene, disease, or drug that actually anchors the
question, rerun with that typed flag before you spend time paginating a noisy
keyword-only result set.

## Avoid keyword reformulation loops

When an agent is iterating on one literature task, pass a short local
`--session` label and request JSON. If the next keyword search overlaps the
previous same-session keyword by at least 60% after BioMCP removes common
search filler words, JSON `_meta.suggestions[]` can point to a better fallback:
inspect the prior hits with `article batch`, map the topic with `discover`, or
narrow by publication year when the current page supports that retry.

```bash
biomcp --json search article -k "Oncotype DX review" --session lit-review-1 --limit 5
biomcp --json search article -k "Oncotype DX DCIS" --session lit-review-1 --limit 5
```

Treat `--session` as a non-secret local correlation label. Do not put PHI,
credentials, email addresses, or user identifiers in it. Markdown article
search output does not show loop-breaker suggestions.

## Start from a known anchor

```bash
biomcp search article -g BRAF --limit 10
```

`search article` always works without credentials. BioMCP keeps
`sort=relevance` as the default, but the effective ranking mode depends on the
query: keyword-bearing searches default to hybrid scoring, while entity-only
searches default to lexical directness. The default source set stays PubTator3,
Europe PMC, PubMed, and compatible Semantic Scholar; use `--source
semanticscholar` or `--source litsense2` explicitly when you want one of those
sources alone. `S2_API_KEY` upgrades Semantic Scholar
requests to authenticated quota; without it, BioMCP uses the shared pool.
BioMCP also caps each federated source's contribution after deduplication and
before ranking. Default: 40% of `--limit` on federated pools with at least
three surviving primary sources. Rows count against their primary source after
deduplication. Use `--max-per-source <N>` to override that cap, use
`--max-per-source 0` for the default cap explicitly, and set it equal to
`--limit` to disable capping.

## Find papers for one exact variant

When the question starts with a specific allele, use the variant pivot rather
than manually trying one spelling at a time:

```bash
biomcp variant articles "BRAF p.V600E" --limit 10
```

The default union resolves the variant once, searches compatible PubTator
annotations and normalized exact aliases, adds validated source citations,
then merges, ranks, and paginates once. JSON preserves each paper's route,
source, and matched-alias provenance. Use `--strategy annotation` or
`--strategy lexical` only to diagnose route recall. If a selected provider is
incomplete, inspect `complete`, `truncated`, `pagination.total`, and
`source_status` rather than treating an empty page as a complete miss.

## Search PubMed directly

```bash
biomcp search article -g BRAF --source pubmed --limit 5
```

Direct PubMed search and the compatible federated PubMed leg apply the same
question-format cleanup before ESearch, so a keyword question can still echo
as written while PubMed receives content terms.

## Add disease context

```bash
biomcp search article -g BRAF -d melanoma --limit 10
```

## Tune semantic versus lexical balance

```bash
biomcp search article -k "Hirschsprung disease ganglion cells" --ranking-mode hybrid --weight-semantic 0.5 --weight-lexical 0.2 --limit 5
```

Use `--ranking-mode lexical` to force the old directness comparator on a
keyword query, `--ranking-mode semantic` to sort by the LitSense2-derived
semantic signal first, or `--weight-*` flags to retune the default hybrid
formula `0.4*semantic + 0.3*lexical + 0.2*citations + 0.1*position`. Rows
without LitSense2 provenance contribute `semantic=0` in semantic-aware
ranking modes.

## Cap one source explicitly

```bash
biomcp search article -k "Kartagener syndrome ciliopathy" --limit 50 --max-per-source 10
```

## Constrain by date

```bash
biomcp search article -g BRAF --since 2024-01-01 --limit 10
```

## Exclude preprints when supported

```bash
biomcp search article -g BRAF --since 2024-01-01 --no-preprints --limit 10
```

## Pull the full-text section

```bash
biomcp get article 22663011 fulltext
```

## Fetch several shortlisted papers at once

```bash
biomcp article batch 22663011 24200969 39073865
```

Use `article batch` after search when you already know the candidate PMIDs or
DOIs and want compact title/journal/year/entity cards before opening one paper
in full detail. The helper preserves input order and still works when
`S2_API_KEY` is unset.

## Use `--type` carefully

```bash
biomcp search article -g BRAF --type review --limit 5
```

`--type` on the default `--source all` route uses Europe PMC + PubMed when the
other selected filters are PubMed-compatible. If you also need
`--open-access` or `--no-preprints`, PubMed drops out and the search collapses
to Europe PMC-only with an explicit note. Use `--source pubmed` when you want
PubMed-only article search on the compatible filter set and do not need those
PubMed-incompatible filters.

## Inspect the ranking rationale in JSON

```bash
env -u S2_API_KEY biomcp --json search article -g BRAF --limit 3
```

Look for `semantic_scholar_enabled`, row-level `matched_sources`, and
`ranking` metadata to see why a paper ranked where it did. Hybrid rows expose
normalized semantic, lexical, citation, and source-position components plus the
composite score; lexical rows preserve the existing directness metadata.

## Inspect the executed search plan

Markdown:

```bash
env -u S2_API_KEY biomcp search article -g BRAF --debug-plan --limit 3
```

JSON / MCP-friendly text output:

```bash
env -u S2_API_KEY biomcp --json search article -g BRAF --debug-plan --limit 3
```

`--debug-plan` adds a top-level `debug_plan` payload in JSON and prepends the
same payload as a fenced JSON block in markdown. Request JSON+plan for MCP
callers with `--json --debug-plan`.

## Follow-up pattern

After identifying key papers, pivot to trials or variants:

```bash
biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 5
biomcp search variant -g BRAF --limit 5
```
