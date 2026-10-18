---
priority: 3
---

# 2184 — Route an rsID in plain article search through the PubTator entity

Status: OPEN.

Milestone: 0.9.4

## Outcome

`search article <rsID> --source pubtator` uses the PubTator variant entity, as `variant articles` already does.

- Evidence: research's spot check: `search article rs113488022 --source pubtator` returns the 47 text matches, while the variant-articles annotation route finds about 1,250 candidates.
- Changes: `src/entities/article/query.rs` `tests/test_article_rsid_search.py` `CHANGELOG.md`. Reuse the existing typed PubTator resolver for exact rsID keywords.
- Proof: recorded replies show the plain search resolving the rsID to its entity; current main uses text search.

## Progress

- 2026-10-18 started

Coordinator pre-code scope: exact claim `src/entities/article/query.rs`, `tests/test_article_rsid_search.py`, `CHANGELOG.md`. Current build_pubtator_query appends an exact rsID keyword literally; the existing variant resolver already verifies the matching PubTator variant entity using recorded db_id. Reuse that resolver for exactly-rsID plain article keywords without changing variant-article verification, alias rules or gene fallback. Existing recorded autocomplete for rs113488022 proves the typed entity despite an inconsistent display label. Add an isolated real-CLI fixture case asserting the autocomplete request and entity-valued search request; also cover absent/mismatched replies retaining current literal query behavior and ordinary text remaining untouched. Run the actual outside-in case against current built main to capture RED before production correction, then targeted checks. No new provider or biological interpretation. Build/test jobs remain held until the current main check and2191idle baseline complete.

- 2026-10-18 next: Queued with isolated four-case CLI test prepared and production unchanged; no jobs or source push. PM priority1 ORCID registry fix2216 takes this active lane slot first. Resume this priority3 lane when a slot frees; preserve prepared work.
