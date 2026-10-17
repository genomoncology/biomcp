---
priority: 3
---

# 2081 — Delete every test that is not one of the four kinds

Status: OPEN.

Milestone: 0.9.3

## Outcome

Retained routine tests protect behavior, edge cases, external contracts, fixed defects or refusals; redundant construction/restatement tests and unused test-only code are removed in reviewed small batches.

Depends on: 2156

- Starts from: existing reviewed deletion batches and their outside-in replacements. Preserve their work rather than restarting it.
- Keeps: caller behavior and refusal regressions, parser/resolver edge cases, stress-filter targets, search all, update and uninstall. Do not delete the only test of a required behavior.
- Changes: claims `CHANGELOG.md`. Before each batch, name its actual files and avoid active product claims. Remove redundant tests and their unused test-only items. Remove avoidable waits with clocks, signals or bounded fixtures; remove sleep allowances when their final legitimate use disappears.
- Proof: each batch names the removed claims and their surviving outside-in tests, reports counts/compile time/routine duration and coverage without a new coverage threshold, and passes the routine merged-candidate gate through the build job. The existing article CLI and drug-label deletion branch must execute its replacements in an active lane before landing: adding claims only to omitted spec pages does not qualify. Cover update/check/uninstall through a staged local release before deleting their owning tests.
- Defers: runner warning behavior to 2095; extra golden recordings to 2079; broad fixture restructuring to the next release.

PM decision: 2026-10-17. Remove the hard ten-second acceptance bar. Long tests belong in the long suite; elapsed time alone never fails the routine runner. Require only replacement claims affected by each deletion, not blanket new coverage or reopened completed batches. Search-all prerequisites use the existing executed golden cases.
