---
priority: 1
---

# 2177 — Keep fixture output out of repository scans

Status: OPEN.

Milestone: 0.9.3

## Outcome

Fulltext replay fixtures create temporary server and cache state inside ignored test output, so concurrent repository scans never read files that fixture cleanup can remove.

- Starts from: main beb08a9f3 and the existing fulltext fixture entry point.
- Changes: claims `spec/fixtures/setup-article-fulltext-source-fixture.sh` `tests/test_fulltext_fixture_output_location.py` `CHANGELOG.md`. Request an expansion before changing other files.
- Keeps: recorded replies, exact-loopback authorization, Unicode cache paths, cleanup, bounded child lifecycle and the existing repository audit coverage of tracked and new source files.
- Proof: invoke the real fixture entry point on current main and show its generated paths appear in the untracked repository source inventory. After the fix, the same generated state belongs to the existing ignored test-output area, stays absent from that inventory while the fixture is live, and is removed by normal cleanup. Keep source scanning intact. Run the affected fixture and audit contracts through the build job.
- Defers: fixture identifier refactoring, broad audit rewrites and unrelated process changes.

## Evidence and implementation brief

Gate 1792260718966-18437 ran make landing-gate on the integrated supplement candidate. The Python quality-wrapper contract failed in check-zero-coupling.py while read_bytes opened a temporary fulltext fixture cache lock that concurrent cleanup had removed. The path was beneath an untracked biomcp-rmcp-fulltext directory at the repository root. That fixture directory layout also exists on current main. The gate separately found two supplement worker helper conventions; those remain in 2152.

Restated outcome: runtime fixture files must never enter the repository source scan. Reproduce their visibility through the actual fixture command before changing its output location. Prefer the existing ignored test-output directory; do not suppress audit errors or hide tracked source. Name the exact controlled red command and result before the product edit. Keep this repair small and independent of supplement extraction code.
