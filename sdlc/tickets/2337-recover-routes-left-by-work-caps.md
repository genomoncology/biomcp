# 2337 — Recover routes left unattempted by work caps

Status: OPEN.

Milestone: 0.9.4

## Outcome

Variant literature searches attempt the routes that alias_limit and logical_work_cap now leave unattempted, within provider rate limits and the command deadline, and merge their results. Each cap that still binds is reported with the routes it left and why. Coordinate with 2327, which recovers dropped coding spellings: one recovery mechanism serves both.

- Proof: outside-in replay tests for the study's three capped cases show the formerly unattempted routes attempted and merged, complete=true when nothing is left, and a case past the cap reporting each remaining route.

## Evidence

- On 2026-10-21, research ran five fresh agents with BioMCP 0.9.4 (commit cdb9f4a1b) on literature searches for five hereditary cancer variants, with no answer key. Exact commands and output are in research's protected store under runs/0010/five-v1.
- Item 4: alias_limit and logical_work_cap left routes unattempted in three cases, and the results said so. This is the same disclosure-without-recovery gap as research's D1, which 2327 fixes for coding spellings.
- logical_work_cap lives in src/entities/article/variant_search.rs.
- Ian asked on 2026-10-21 to clear every known issue in 0.9.4.
