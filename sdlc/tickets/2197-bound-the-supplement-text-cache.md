---
priority: 2
---

# 2197 — Bound the supplement text cache

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 343b394, accept

## Outcome

The supplement text cache obeys the same age, size and disk limits as the rest of the cache, shows in `cache stats`, and reads without walking its directory.

- Evidence: release review of 7025e117: `sessions/article-supplement-text-v1` (from 2152) stores up to 16 MiB per file with no expiry; `cache clean --max-age` skips it, `cache stats` omits it, the size and disk limits ignore it, and each read walks the directory.
- Changes: `CHANGELOG.md`, `docs/user-guide/supplementary-materials.md`, `src/entities/article/supplement_search.rs`, `src/entities/article/supplement_search/cache.rs`, `tests/article_asset_terminal.rs`, `tests/article_asset_terminal/supplement_cache.rs`. bring it under the managed cache's limits and accounting, or store it through the existing HTTP cache path.
- Proof: outside-in cases: clean by age removes old entries, the size limit evicts them, stats count them, and a lookup reads one entry by key. RED job2197/1792322315264-77261 failed on the absent managed extraction key. Behavioral job2197/1792323370603-17371 passed nine supplement tests before unused-mut lint failure; final job2197/1792323604406-2239 passed formatting, all-target lint and quality after removing that binding qualifier. Retired unreleased session entries remain until existing whole-cache clear; persistent totals use2195.
