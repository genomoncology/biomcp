# 2369 — Supplement docs agree on asset keys

Status: OPEN.

Milestone: 0.9.4

Priority: 1

## Outcome

The article user guide and supplementary-materials guide describe challenge and unavailable rows the way the runtime emits them.

## Evidence

- Research recheck of the 0.9.4 candidate b56138d27 (research repo f1c305c, sdlc/records/0042-test-the-0-9-4-candidate-hard.md; raw outputs on Blue under /home/ian/workspace/data/research/runs/0042/), relayed by Jarvis in m3902. These are observations: reproduce on current main first, and withdraw with evidence if it does not reproduce.
- DOC-D9: docs/user-guide/article.md:356-357 says challenge rows have no asset_key. supplementary-materials.md:133-137 and the runtime give an unavailable-asset key without a handle.

## Changes

- Changes: `docs/user-guide/article.md`, `docs/user-guide/supplementary-materials.md`, `CHANGELOG.md`, `sdlc/tickets/2369-supplement-docs-match-asset-key.md`.

## Proof

- Both pages state the runtime behavior, and a doc contract test or example fixture holds them to it.
- Lands before the final 0.9.4 requalification and rehearsal; research rechecks this item on the final build.
