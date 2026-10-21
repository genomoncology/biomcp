---
flow: build
priority: 10
---

# Refuse duplicate completion-record identities

## Goal

Every completion record has one unique four-digit identity, and the routine repository gate rejects a duplicate before it can land. Two different completed changes occupied record 1160, while the task reader reduced both paths to one identity membership and could not diagnose the collision.

At `origin/main` commit `b2e05326`, these files both existed:

- `sdlc/records/1160-keep-provider-trial-titles-inert-in-displayed-article-search-commands.md`
- `sdlc/records/1160-carry-biodata-reference-values-through-the-working-trial-command.md`

There were 734 numbered record files but only 733 unique record IDs. `sdlc/project/tasks` stores record IDs in a set used only to mark tickets done, so it cannot distinguish or report two record paths carrying one identity. `sdlc/project/health` checks ticket and legacy-issue collisions but not record-versus-record collisions.

The project scripts are adoption-pinned byte-for-byte to the canonical SDLC package. This repository does not fork those scripts to repair a BioMCP ledger mistake.

## Required correction

Keep the earlier provider-title shell-safety record at 1160. Rename the newer BioData reference-integration record to 1169 as a 100% Git rename with byte-identical content. Record 1166 remains the separately landed clinical-trial capability contract. IDs 1161–1165 and 1167 are reserved by the reviewed local drafts, and 1168 belongs to this repair.

Add one deterministic BioMCP repository contract that enumerates direct Markdown children of `sdlc/records/`, accepts only the existing `NNNN-*.md` filename shape, parses the four-digit prefix, groups paths by ID, and fails with every duplicate ID and its sorted paths. Nested files do not participate. A temporary-tree regression creates at least two different duplicate-ID groups and one unique control in deliberately unsorted creation order, then asserts the exact complete diagnostic ordered by ID and path. A clean real-tree assertion after the rename proves repository-wide uniqueness.

## Result

The earlier provider-title shell-safety record remains 1160. The newer BioData reference-integration record is now 1169 with byte-identical content. A BioMCP-owned repository contract reports every duplicate completion-record ID and its direct-child paths in stable order while ignoring nested and malformed-name files. The clean real-tree assertion now proves all 735 numbered completion records have unique IDs.

Focused duplicate-record coverage passed 2 tests, the complete documentation-consistency module passed 18 tests, and the independent combined review run passed 48 tests. `git diff --check` passed. The BioData record's old and new blobs both have SHA-256 `a52149857285252d24123e5243044934e8bcb7ce4665aaddf5812f2d07433e9d`. No canonical SDLC project script changed.

Primary-agent integration gates passed: `make lint`; `make test` with 3,152 Rust tests passed (30 skipped), 901 Python tests passed (3 skipped), and the strict documentation build; and `make spec`, including 8 static contract checks. The focused documentation-consistency module also passed all 18 tests on the integrated tree.

## Review

- Design review: accepted after requiring a byte-identical rename, a complete stable diagnostic over multiple duplicate groups, direct-child scope, and no changes to adoption-pinned project scripts.
- Code review: accepted with no remaining findings. Independent review confirmed the exact rename, deterministic temporary-tree and real-tree coverage, complete diagnostics, unchanged canonical-script hashes, and clean diff.

## Boundary

This ticket repairs completion-record identity and adds a BioMCP-owned prevention gate. It does not change ticket status semantics, renumber the earlier 1160 record or record 1166, edit canonical SDLC project scripts, rewrite record bodies, revive completed tickets, or change runtime BioMCP behavior.
