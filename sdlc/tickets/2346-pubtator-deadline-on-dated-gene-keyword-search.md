# 2346 — Finish dated gene and keyword PubTator searches

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 2beae50a0ed124d3ab0956241829e3e7eaf57c1b, accept

## Outcome

A dated gene plus keyword article search completes its PubTator3 leg, or returns the other sources' results and names PubTator3 as timed out with the request it sent. Find why the leg exceeds the 12-second source guard when a simple PubTator3 search answers at once: query shape, page size, pagination or serial requests. Fix the cause within provider limits; do not just raise the guard.

- Proof: an outside-in test of the study command against recorded PubTator3 replies completes within the guard; a timeout case still returns partial results naming PubTator3.

## Evidence

- On 2026-10-21, research reran five hereditary cancer variants with blind agents on qualified BioMCP 0.9.4, with no search cap and direct provider calls allowed. Evidence: research repo commit 561cb47, sdlc/records/0041-tool-issues.tsv (27 observations with exact commands) and 0041-direct-only-findings.tsv (56 direct-only traces). These are observations, not verified defects; confirm on current main first and withdraw with evidence if it does not reproduce.
- Item 4: `biomcp --json search article -g CHEK2 -k 'G167R OR Gly167Arg' --date-from 2026-09-01 --limit 50 --max-per-source 50 --debug-plan` hits a 12-second timeout, and an exact retry does too. A simple PubTator3 search answers HTTP 200 from Blue and the Beelink.
- 2336 added one retry for transient failures. It does not cover a leg that is slow every time.

- Coordinator restatement: preserve gene AND grouped keyword constraints within the existing source guard, retaining other-source results and safe attempted-request details on timeout. Current-main Gator j4159 reproduced the exact study command PubTator timeout. Actual malformed `@GENE_CHEK2 G167R OR Gly167Arg` returned 17471 hits/699 pages in 0.315s; `@GENE_CHEK2 AND (G167R OR Gly167Arg)` returned 15 hits/one page in 0.291s. Query precedence causes unnecessary pagination; do not raise guard or invent date/pagination repairs. j4170 retained credential-free response fixtures. Developer owns query.rs and focused recorded CLI regression, fixtures, feature mapping and changelog; report any additional actual-send diagnostic files before editing.
## Changes

- `src/entities/article/query.rs`: join independently supplied PubTator filters with explicit conjunction and group multiword operands without changing a standalone keyword query.
- `src/sources/mod.rs` and `src/sources/literature_retry.rs`: retain only the admitted PubTator request's path and allowlisted non-authentication query fields for failure reporting.
- `src/entities/article/search.rs` and `src/entities/article/backends.rs`: include that request in a failed source or a later-page degradation while preserving answered sources and rows.
- `tests/test_article_pubtator_deadline.py`, `tests/fixtures/article/pubtator-dated-query/` and `tests/feature-links.json`: exercise the study command against captured provider replies and held initial/later-page requests, including credential exclusion.
- `CHANGELOG.md`: describe the correction.

- Actual query regression j4176 failed both precise-query assertions before the fix; j4180 passed after explicit conjunction/grouping. Diagnostic RED j4189 retained precise-query success but failed initial/later-page safe-request assertions. Final affected j4205 passed on ff4e1c04e plus exact product patch SHA256 41c2c4f9acccfdebe41c4da57bd47216cbf0431d49aaba10e55a23b7c66141be: 61 CLI cases, 94 Rust cases, Ruff and formatting. Coherent candidate2beae50a0ed124d3ab0956241829e3e7eaf57c1b adds only changelog, Changes claim and fixture explanation to the tested source. Latest main3b73f04 differs from the tested base only in SDLC.
- Fresh independent reviewer accepts 2beae50a0ed124d3ab0956241829e3e7eaf57c1b after whole-diff/shared-caller review of query semantics, unchanged standalone queries, transport-admitted allowlisted diagnostics, credential exclusion and initial/later-page partial results. No changes required. No guard, date, pagination, retry allowance or pacing policy changed. Yellow ticket-evidence/2346/biomcp-2346-evidence.tar.gz SHA256 c805560d961f8ffc19cc19575d451c28974f7d9c2cb6df2fce03fae17944fc9d retains all seven executed proof logs, bounded recipes and exact tested patch. j4157 was cancelled before execution. Batch gate remains required.
