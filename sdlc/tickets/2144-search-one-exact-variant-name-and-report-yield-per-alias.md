---
flow: build
priority: 3
deps: [2130, 2138]
---

# 2144: Search one exact variant name and report yield per alias

Status: OPEN.

Milestone: 0.9.3

## Outcome

A caller can search literature for one exact variant name with no alias expansion, and can see how many papers each alias found from each source, so a scorer attributes yield per name.

## Evidence

Asked 2026-10-16 by the research team, who score variant literature yield per name.

- `variant articles` has no exact-name route. Every strategy resolves the variant and searches up to 10 aliases (`exact_aliases`, `src/entities/article/variant_search.rs:1643-1724`); `--strategy lexical` still sends every alias.
- `search article -k NAME` searches one string unexpanded across PubTator, Europe PMC, PubMed and Semantic Scholar, but no source receives it as a phrase (`src/entities/article/query.rs:22-37`, `:225-243`, `:594-631`).
- Each `variant articles --json` row carries `provenance[]` with route, source, query aliases and `matched_alias`. But `matched_alias` is only the alias sent (`variant_search.rs:1361`, `:1947`), only the visible page (at most 50) is attributed, `debug_plan.candidate_trace` carries the route alone, and each alias call stops at 25 results with no count of what was cut.

## Work

1. Add an exact mode: `variant articles --strategy exact NAME` searches the caller's string with no resolution or expansion, phrase-quoted in each provider's own syntax, and reports which sources it reached.
2. Add a per-alias, per-source yield table to `--debug-plan`: hits received, hits kept after merging, and whether the 25-result cap cut the list.

## Acceptance

1. Recorded-reply tests show the exact mode sends one phrase-quoted query per source and no alias.
2. A recorded-reply test of a two-alias search shows each alias's per-source counts in the debug plan, including a cut list.
3. The changelog carries a bullet for this ticket.

## Reproduction on main

Run 2026-10-17 on the macOS gate host at `f9a11e81e` (origin/main tip), pm job
`1792211931779-13479`, live providers, `--no-cache`.

Gap 1 — no exact-name route:

```
biomcp --json variant articles "BRAF V600E" --strategy exact --limit 5 --no-cache
```

exits 2 with `invalid value 'exact' for '--strategy <STRATEGY>' [possible
values: union, annotation, lexical]`. Every existing strategy resolves the
variant and expands up to 10 aliases; none searches the caller's string alone.

Gap 2 — unattributed yield. The same variant with the lexical strategy and a
debug plan:

```
biomcp --json variant articles "BRAF V600E" --strategy lexical --limit 5 --debug-plan --no-cache
```

exits 0 with 5 rows. The debug plan's top-level keys are `budgets,
candidate_trace, counts, deadline, next, normalized_aliases, provider_queries,
ranking, routes, stopped_routes, truncated, work_allocation` — no per-alias,
per-source yield anywhere (`any per-alias yield key anywhere: False`). The run
searched several aliases (`provider_queries` carries `strict` legs for
`BRAF V600E`, `BRAF V207E`, and more), the candidate trace carries the route
alone (`identifier, route, provider_terminal_state, received, after_union,
after_dedup, rank_position, verification_disposition,
pagination_disposition`), and each row's provenance `matched_alias` only names
the alias sent (`BRAF p.V600E`, `BRAF c.1799T>A`, `BRAF p.Val600Glu`). Nothing
counts, per alias and source, the hits received, the hits kept after merging,
or whether each alias call's 25-result cap cut the list.
