# 2365 — Variant article paging reaches the end

Status: OPEN.

Milestone: 0.9.4

Priority: 1

- Changes: `CHANGELOG.md`, `docs/how-to/find-articles.md`, `src/entities/article/variant_search.rs`, `tests/test_variant_article_recovery.py`.

## Outcome

Variant article paging returns rows up to the true end of results; has_more is true only when a later page can return rows.

## Evidence

- Research recheck of the 0.9.4 candidate b56138d27 (research repo f1c305c, sdlc/records/0042-test-the-0-9-4-candidate-hard.md; raw outputs on Blue under /home/ian/workspace/data/research/runs/0042/), relayed by Jarvis in m3902. These are observations: reproduce on current main first, and withdraw with evidence if it does not reproduce.
- C003-PAGINATION: `biomcp --json variant articles 'CHEK2 G167R' --limit 50 --offset 110` returns 0 rows with has_more=true and complete=false, while all 30 work items are ok.

## Proof

- Outside-in replay test: paging the CHEK2 G167R fixture to its end returns every row once, and the last page reports has_more=false.
- Lands before the final 0.9.4 requalification and rehearsal; research rechecks this item on the final build.
