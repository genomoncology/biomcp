---
flow: build
priority: 2
deps: []
---

# 2145: Mark an empty article search incomplete when a source is down

Status: OPEN.

Milestone: 0.9.3

## Outcome

`search article` follows the documented rule that `variant articles` already follows: a zero-row result while a source was unavailable reports `complete: false`, error `source_unavailable`, and exits 1; an empty search where every source answered stays an ok result with exit 0.

## Evidence

Reported 2026-10-16 by the research team.

- Without `S2_API_KEY`, `biomcp --json search article -g ABRAXAS1 -k p.G39V --limit 50` exits 0 with count 0 and no `complete` or `error` field, while `source_status` lists Semantic Scholar as unavailable.
- The rule is documented for variant article search in `docs/how-to/find-articles.md:109-113` and `docs/user-guide/cli-reference.md:863-869`, and coded at `src/entities/article/variant_search.rs:3473`.

## Work

- Reproduce on main first.
- Give `search article` JSON a `complete` field and the same zero-row error and exit rule. A result with rows and a down source stays exit 0 with `complete: false`.
- State the rule for `search article` in the same two documents.

## Acceptance

1. A recorded-reply test with Semantic Scholar unavailable and no rows exits 1 with `source_unavailable`; it fails on current main.
2. A recorded-reply test where every source answers with no rows exits 0 with `complete: true`.
3. The changelog carries a bullet for this ticket.
