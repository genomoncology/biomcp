---
priority: 1
---

# 2159 — Derive package membership without a manual count

Status: COMPLETE.

Milestone: 0.9.3

Depends on: 2156

Reviews: revision ff0b41e, accept

Reviews: revision 8f13b95, accept

Reviews: revision 7767751, accept

Reviews: revision e75e7a4, accept

Landed: e75e7a4 (fast-forward: Accepted source landed inside carrier 27e59b197 with the passing shared batch gate; record this carried ticket separately without rerunning unchanged code.)

## Outcome

Package boundary checks detect missing required files and unexpected packaged files without maintaining an exact file-count constant.

- Starts from: `tests/test_source_package_boundary.py` and `Cargo.toml`. Landing 2124 added a packaged test without updating the parallel count; `74dfc40e9` raised the number but left the duplication.
- Keeps: the runtime package boundary, source-versus-archive membership comparison, forbidden build/local output exclusions and the release-only cold packaged build.
- Changes: claims `tests/test_source_package_boundary.py` `Cargo.toml` `CHANGELOG.md`. Change the manifest only if the actual declared boundary needs correction.
- Proof: reproduce the maintained-count defect with a legitimate member fixture, then show that allowed additions require no parallel count edit, missing required members fail, unexpected boundary members fail and the archive membership agrees with the declared package list. Run targeted boundary checks through the build job; cold package construction stays in the release suite.
- Defers: unrelated package restructuring.

PM decision: 2026-10-17. Ian approved removing this small source-of-truth duplication for 0.9.3. Derive membership from the declared package boundary; do not replace the count with another hand-maintained whole-file inventory.

## Progress

- 2026-10-17 started
- 2026-10-17 next: Batch members: 2159, 2149, 2168, 2128, 2122 and 2171. Shared landing gate: job 1792250897357-93113, candidate ea4161cc814ad034634ffd249e67b475e77e38dc. Required preceding main check 1792249698624-96397 passed. Gate is running; each ticket keeps its individual review and landing commit.
- 2026-10-17 next: Batch members: 2159, 2149, 2168, 2128, 2122 and 2171. Shared configured landing gate PASSED: job 1792251400030-83132, candidate c264da2a525c2c6e5a424693d55dfab0f59fadf4, 240.3 seconds. This replaces the earlier failed batch attempts. Preceding full main check 1792249698624-96397 passed. Land each reviewed member through pm, then run one full check on final batch main.
