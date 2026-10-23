# 2367 — Union and legacy strategies finish their first page

Status: OPEN.

Milestone: 0.9.4

Priority: 1

## Outcome

The union and legacy variant article strategies attempt every planned work item for the first page within the invocation deadline, or plan fewer items so the page completes.

- Changes: `src/entities/article/variant_search.rs`, `tests/test_variant_article_recovery.py`, `tests/test_variant_article_first_page.py`. Run independent planned provider work concurrently and exercise first-page accounting.

## Evidence

- Research recheck of the 0.9.4 candidate b56138d27 (research repo f1c305c, sdlc/records/0042-test-the-0-9-4-candidate-hard.md; raw outputs on Blue under /home/ian/workspace/data/research/runs/0042/), relayed by Jarvis in m3902. These are observations: reproduce on current main first, and withdraw with evidence if it does not reproduce.
- Union and legacy strategies for ABCC8 R1436Q and R1437Q, CDKN2A A57V and BRCA1 P1749R stop the first page under invocation_deadline with 11 to 23 work items not attempted. The output discloses this; it is not a cap omission. Find whether the time goes to serial waits, a slow provider, or more work than the deadline can hold.

## Proof

- Route-level timing names the cause. Outside-in replay test: the four cases complete their first page with all work items attempted under the deadline.
- Lands before the final 0.9.4 requalification and rehearsal; research rechecks this item on the final build.
