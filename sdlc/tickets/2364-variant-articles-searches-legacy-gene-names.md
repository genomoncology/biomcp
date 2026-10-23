# 2364 — Variant article search uses legacy gene names

Status: OPEN.

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
