# Review — T015 Unified Multi-Source Article Search

## Status
- success

## Context Reviewed
- `.climb/design.md`
- `.climb/dev-log.md`
- Code changes in:
  - `src/entities/article.rs`
  - `src/sources/pubtator.rs`
  - `src/transform/article.rs`
  - `src/cli/mod.rs`
  - `src/cli/search_all.rs`
  - `src/render/markdown.rs`
  - `templates/article_search.md.j2`
  - `src/cli/list.rs`
  - `src/cli/list_reference.md`

## Quality Gates
- `cargo fmt --check`: pass
- `cargo build`: pass
- `cargo test`: pass (387 passed)
- `cargo clippy -- -D warnings`: pass
- `make spec`: not applicable (`spec/` directory and `spec` target are absent)

## Review Checklist
- Code quality: pass
- Test quality: pass (plus added regression coverage during review)
- Cleanup and consistency: pass
- Edge cases and error handling: pass

## Issues Found and Fixed During Review
1. Federated graceful-degradation behavior (AC4) lacked direct unit-level regression coverage, making refactors risky.
- Fix:
  - Extracted federated merge/fallback logic into `merge_federated_pages(...)` and reused it from `search_federated_page(...)`.
  - Added defensive truncation to `limit` on single-leg fallback paths.
  - Added regression tests:
    - `merge_federated_pages_dedups_with_pubtator_priority`
    - `merge_federated_pages_returns_surviving_pubtator_leg`
    - `merge_federated_pages_returns_surviving_europe_leg`
    - `merge_federated_pages_returns_first_error_when_both_fail`

2. Article search markdown help text for `--type` values was incomplete (AC8 docs/output polish).
- Fix:
  - Updated `templates/article_search.md.j2` to list full supported type aliases:
    - `research-article|research|review|case-reports|meta-analysis`

## AC4–AC6 / AC8 Verification Notes
- AC4 (parallel fan-out + graceful degradation): verified in code path (`tokio::join!`) and now backed by new merge/fallback unit tests.
- AC5 (source-specific pagination/filter correctness): verified orchestrator routing + source-specific page math in `search_europepmc_page` and `search_pubtator_page`.
- AC6 (`search all` uses entity-ranked search): verified in `src/cli/search_all.rs` (`ArticleSort::Relevance` + `ArticleSourceFilter::All`).
- AC8 (output/docs): verified grouped source rendering and docs; template/help updated.

## Required Self-Review Answers
- How would you build this differently now that it's done?
  - I would separate backend orchestration from network I/O earlier so failure/merge/pagination semantics are fully pure-tested without requiring structural extraction during review.
- Are you proud of this work?
  - Yes, after review fixes; behavior now has stronger regression protection around the highest-risk federated paths.
- Did you discover any code smells?
  - Minor: federated pagination totals remain intentionally unknown, which limits strict `has_more` certainty in deduplicated combined windows; acceptable for this ticket but worth documenting further if UX expectations tighten.

## Handoff for Verifier
- Re-run full gates in this worktree; all should be green.
- Verify new tests in `src/entities/article.rs` around `merge_federated_pages_*`.
- Spot-check article markdown footer/help output in `templates/article_search.md.j2` for updated `--type` guidance.
