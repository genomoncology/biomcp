# 2084 — Make Europe PMC offset pages line up

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 1691261, accept

## Outcome

For a Europe PMC relevance search, `--limit 5` and then `--limit 5 --offset 5` return the same ten rows, in the same order, as `--limit 10`.

## Evidence

Split from 2066 on 2026-10-14. Found by the post-release install and product test of v0.9.2.

- `--json search article pembrolizumab --source europepmc --limit 5` and the same with `--offset 5` both return PMID 42805115 fifth, and PMID 42786043, fifth in a single `--limit 10` search, appears on neither page.
- Cause: `search_relevance_page` fetches `offset + limit` rows, adds Semantic Scholar citation counts to that whole pool, ranks it and then slices. A 5-row pool and a 10-row pool rank differently, so the pages do not line up. Changing how the offset is counted would not fix it.
- PM decision 2026-10-14: rank a pool whose size is `offset + limit` rounded up to a multiple of 25, then slice. Pages then line up within each block of 25 rows, at no extra provider request and at most one Semantic Scholar batch call per 25 rows. A page that straddles a 25-row boundary can still differ, and the changelog bullet says so. Keeping Europe PMC's own order for paged requests was rejected because it drops the local ranking.

- Starts from: `src/entities/article/search.rs`, `search_relevance_page` (line 664), which fetches `limit + offset` rows; `src/entities/article/enrichment.rs:415`, which adds citation counts to the pool, ranks it and slices; `EUROPE_PMC_PAGE_SIZE = 25` (`src/entities/article/mod.rs:1096`), so the provider already returns 25 rows per request.
- Keeps: the local citation ranking; every other source's paging.
- Changes: claims `src/entities/article/search.rs` `src/entities/article/search/tests.rs` `testdata/sources/europepmc/**` `testdata/sources/semantic_scholar/**` `testdata/sources/capture-receipts.json` `docs/user-guide/article.md` `CHANGELOG.md` `tests/test_docs_changelog_refresh.py`. Apply the rounded pool size in both single-source arms of `search_relevance_page`; document the 25-row block in `docs/user-guide/article.md`; one Fixes changelog bullet that names the boundary case.
- Proof: an offline test with a recorded 25-row Europe PMC reply and a recorded Semantic Scholar reply shows limit 10 equals limit 5 followed by offset 5; it fails on current main.
- Defers: nothing.
- Order: safe alongside anything.
