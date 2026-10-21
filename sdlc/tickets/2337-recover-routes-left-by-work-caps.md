# 2337 — Recover routes left unattempted by work caps

Status: OPEN.

Milestone: 0.9.4

## Outcome

Variant literature searches attempt the routes that alias_limit and logical_work_cap now leave unattempted, within provider rate limits and the command deadline, and merge their results. Each cap that still binds is reported with the routes it left and why. Coordinate with 2327, which recovers dropped coding spellings: one recovery mechanism serves both.

Restated outcome: recover papers from compatible routes that the existing planner omits, while preserving previously selected queries, identity checks, provider pacing and truthful incomplete results.

Changes: `src/entities/article/variant_search.rs`, `src/entities/article/enrichment.rs`, `tests/test_variant_article_recovery.py`, `tests/fixtures/article/recovery/authored.json`, `tests/feature-links.json`, `CHANGELOG.md`, and this ticket. Add a small shared Europe PMC query helper only if the existing endpoint requires it; update this claim before that edit.

- Proof: outside-in replay tests for the study's three capped cases show the formerly unattempted routes attempted and merged, complete=true when nothing is left, and a case past the cap reporting each remaining route.

## Evidence

- On 2026-10-21, research ran five fresh agents with BioMCP 0.9.4 (commit cdb9f4a1b) on literature searches for five hereditary cancer variants, with no answer key. Exact commands and output are in research's protected store under runs/0010/five-v1.
- Item 4: alias_limit and logical_work_cap left routes unattempted in three cases, and the results said so. This is the same disclosure-without-recovery gap as research's D1, which 2327 fixes for coding spellings.
- logical_work_cap lives in src/entities/article/variant_search.rs.
- Ian asked on 2026-10-21 to clear every known issue in 0.9.4.
- PM decision m3100 authorizes batching and request reuse first. If their resulting work still exceeds the discovery allowance, derive one explicit named allowance from the planned provider units. Preserve the exact lexical and identity allowances, provider pacing and invocation deadline. Report remaining omissions by route and measure CHEK2 request counts and elapsed time before and after on the same machine.
- Fresh design review accepted one stored per-invocation plan shared by execution, debug output, coverage and omissions. Retain the existing lexical list and selected discovery queries. Group only overflow PubMed/Europe PMC clauses with their gene/change pairing intact; a saturated page makes every group member incomplete. Reuse the actual requested PubTator entity calls without claiming each alias spelling was queried. Validate exact requested PMID and MED source before supplying batched Europe PMC hints; retain PubTator-first resolution and explicit missing/provider-failure outcomes. The proposed allowance of 150 was withdrawn because it lacked a derivation.
- Accepted-2327 baseline reproduction: M5 job j2217 at d3ecf85fcea35d6a353316130bd304cdaeb1da24 ran the isolated authored four-case outside-in payload after `make prepare-test-contracts`; pytest exited 1 with four failures at the missing target-paper assertions and no fixture misses. This run covers the accepted 2327 source rather than the later main composition. The commands were `biomcp --json variant articles "BRCA1 p.P1749R" --limit 50 --debug-plan`, `biomcp --json variant articles "CHEK2 p.G167R" --limit 50`, and `biomcp variant articles "ABRAXAS1 G39V" --limit 50 --json`. CHEK2 has separate under-cap and saturated reply controls. Replies are authored controls, not captured provider replies. The saturated control must remain incomplete; it cannot establish complete recovery.
- Current-main reproduction: M5 job j2227 ran the same corrected payload and commands at published main 51d0732129e7136673b1d163ad1686b81ea51010. The job command was `pm job run --machine m5 2337 51d0732129e7136673b1d163ad1686b81ea51010 --disk 8 --memory 7 -- python3 -c <isolated test payload>`, submitted by `/tmp/biomcp-2337-probe/run_red.py`. The payload prepared the checkout's own CLI/Python harness, then ran `pytest -q -s` against the authored four-case test with isolated home/config/cache and no source keys. Terminal output was `4 failed in 4.11s`; all four assertions failed because their target paper was absent, with no fixture misses. The job log retains the exact requests, route omissions and authored-control measurements. Product edits may now begin.
