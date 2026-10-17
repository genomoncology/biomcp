---
priority: 1
---

# 2158 — Reject joined changelog entries

Status: OPEN.

Milestone: 0.9.3

Depends on: 2156

Reviews: revision 1407c25, accept

## Outcome

The changelog gate rejects two entries joined onto one line while accepting correctly separated entries and the existing heading structure.

- Starts from: `scripts/check-changelog-coverage.py` and `tests/test_changelog_coverage_check.py`. Historical repair `18711ac04` joined the 2126 entry to the 2125 entry; `73e091a1a` separated them, but shape validation did not prevent recurrence.
- Keeps: first-parent landing attribution, descriptive entry coverage, duplicate checks and the fixed release headings.
- Changes: claims `scripts/check-changelog-coverage.py` `tests/test_changelog_coverage_check.py` `CHANGELOG.md`.
- Proof: through the checker command, a fixture with two formerly valid entries joined without their newline fails before implementation; the same entries separated normally pass. Run focused checker contracts through the build job and the routine landing gate.
- Defers: a new changelog writer or broader release framework.

PM decision: 2026-10-17. Ian approved this small prevention fix for 0.9.3. The separate missing-entry attribution defect is covered by 2125; do not reopen it.
