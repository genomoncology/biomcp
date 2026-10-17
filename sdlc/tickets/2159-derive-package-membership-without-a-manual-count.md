---
priority: 1
---

# 2159 — Derive package membership without a manual count

Status: OPEN.

Milestone: 0.9.3

Depends on: 2156

Reviews: revision ff0b41e, accept

## Outcome

Package boundary checks detect missing required files and unexpected packaged files without maintaining an exact file-count constant.

- Starts from: `tests/test_source_package_boundary.py` and `Cargo.toml`. Landing 2124 added a packaged test without updating the parallel count; `74dfc40e9` raised the number but left the duplication.
- Keeps: the runtime package boundary, source-versus-archive membership comparison, forbidden build/local output exclusions and the release-only cold packaged build.
- Changes: claims `tests/test_source_package_boundary.py` `Cargo.toml` `CHANGELOG.md`. Change the manifest only if the actual declared boundary needs correction.
- Proof: reproduce the maintained-count defect with a legitimate member fixture, then show that allowed additions require no parallel count edit, missing required members fail, unexpected boundary members fail and the archive membership agrees with the declared package list. Run targeted boundary checks through the build job; cold package construction stays in the release suite.
- Defers: unrelated package restructuring.

PM decision: 2026-10-17. Ian approved removing this small source-of-truth duplication for 0.9.3. Derive membership from the declared package boundary; do not replace the count with another hand-maintained whole-file inventory.
