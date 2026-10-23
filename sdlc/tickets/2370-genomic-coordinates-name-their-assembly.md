# 2370 — Genomic coordinates name their assembly

Status: OPEN.

Milestone: 0.9.4

Priority: 1

## Outcome

Every genomic coordinate BioMCP prints names its assembly, so a GRCh37 position is never read as GRCh38.

## Evidence

- Research recheck of the 0.9.4 candidate b56138d27 (research repo f1c305c, sdlc/records/0042-test-the-0-9-4-candidate-hard.md; raw outputs on Blue under /home/ian/workspace/data/research/runs/0042/), relayed by Jarvis in m3902. These are observations: reproduce on current main first, and withdraw with evidence if it does not reproduce.
- Item 10: CDKN2A Q50P reports chr9:g.21974678T>G while the user supplied NC_000009.12:g.21974679T>G. Ticket 2345 found this correct: clinical review confirmed the GRCh37 coordinate and the GRCh38 one are the same allele. The output omits the assembly, so research and agents read it as a one-base error.

## Proof

- Outside-in test: the CDKN2A Q50P debug plan and variant output label the coordinate GRCh37 (or give both assemblies where known).
- Lands before the final 0.9.4 requalification and rehearsal; research rechecks this item on the final build.
