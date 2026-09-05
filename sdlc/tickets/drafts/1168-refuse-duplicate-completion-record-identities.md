---
flow: build
priority: 10
---

# Refuse duplicate completion-record identities

## Goal

Every completion record has one unique four-digit identity, and the routine repository gate rejects a duplicate before it can land. Today two different completed changes occupy record 1160, while the task reader reduces both paths to one identity membership and cannot diagnose the collision.

At `origin/main` commit `b2e05326`, these files both exist:

- `sdlc/records/1160-keep-provider-trial-titles-inert-in-displayed-article-search-commands.md`
- `sdlc/records/1160-carry-biodata-reference-values-through-the-working-trial-command.md`

There are 734 numbered record files but only 733 unique record IDs. `sdlc/project/tasks` stores record IDs in a set used only to mark tickets done, so it cannot distinguish or report two record paths carrying one identity. `sdlc/project/health` checks ticket and legacy-issue collisions but not record-versus-record collisions.

The project scripts are adoption-pinned byte-for-byte to the canonical SDLC package. This repository must not fork those scripts to repair a BioMCP ledger mistake.

## Required correction

Keep the earlier provider-title shell-safety record at 1160. Rename the newer BioData reference-integration record to 1169 as a 100% Git rename with byte-identical content. Record 1166 remains the separately landed clinical-trial capability contract. IDs 1161–1165 and 1167 are reserved by the reviewed local drafts, and 1168 belongs to this repair.

Add one deterministic BioMCP repository contract that enumerates direct Markdown children of `sdlc/records/`, accepts only the existing `NNNN-*.md` filename shape, parses the four-digit prefix, groups paths by ID, and fails with every duplicate ID and its sorted paths. Nested files do not participate. A temporary-tree regression creates at least two different duplicate-ID groups and one unique control in deliberately unsorted creation order, then asserts the exact complete diagnostic ordered by ID and path. A clean real-tree assertion after the rename proves repository-wide uniqueness.

## Done, observably

- The two completed 1160 behaviors survive under unique IDs: shell safety remains 1160 and BioData reference integration becomes 1169.
- The BioData record keeps its exact Git blob bytes through the rename; no completion-record body is rewritten.
- A deterministic test rejects multiple duplicate record-ID groups, reports every colliding path in stable ID/path order, ignores nested files, and does not report the unique control.
- The existing canonical-adoption hash assertions for `sdlc/project/tasks`, `health`, and provenance remain unchanged and green.
- `sdlc/project/tasks` continues to permit a record whose ticket is absent, as its documented lifecycle contract requires.
- `make lint`, `make test`, and `make spec` pass.

## Boundary

This ticket repairs completion-record identity and adds a BioMCP-owned prevention gate. It does not change ticket status semantics, renumber the earlier 1160 record or record 1166, edit canonical SDLC project scripts, rewrite record bodies, revive completed tickets, or change runtime BioMCP behavior.
