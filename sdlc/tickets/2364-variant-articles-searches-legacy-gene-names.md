# 2364 — Variant article search uses legacy gene names

Status: WITHDRAWN. Does not reproduce on exact main01eccd84ff53cc97c684906d95c730610d51feae: live j6532 shows MyGene supplies SUR1 and five selected provider plans include it. Existing lexical plus default-strategy control j6553 passes20 cases on unchanged product. No product change justified; test-only checkpoint18eddefa2 is not a defect fix. Source-derived alias behavior already meets the outcome.

Milestone: 0.9.4

Priority: 1

## Outcome

Variant article search includes established legacy gene symbols in its provider queries, so papers that use only the old name are found.

## Evidence

- Research recheck of the 0.9.4 candidate b56138d27 (research repo f1c305c, sdlc/records/0042-test-the-0-9-4-candidate-hard.md; raw outputs on Blue under /home/ian/workspace/data/research/runs/0042/), relayed by Jarvis in m3902. These are observations: reproduce on current main first, and withdraw with evidence if it does not reproduce.
- D2: `biomcp --json variant articles 'ABCC8 R1437Q' --limit 5 --debug-plan --verify-identity` still leaves the old name SUR1 out of the selected provider queries.

## Proof

- Outside-in test: the debug plan for ABCC8 R1437Q includes SUR1 in the provider queries, with the alias taken from the gene identity source rather than a hard-coded list.
- Lands before the final 0.9.4 requalification and rehearsal; research rechecks this item on the final build.
