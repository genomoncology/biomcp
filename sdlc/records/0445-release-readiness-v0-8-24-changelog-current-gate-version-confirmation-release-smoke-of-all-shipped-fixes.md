---
base: f324e7f99979e97b4bcbb9819c59132d3467c422
head: ef725872bce73e5c3fd3d74872477291d817241b
---
1. **The CHANGELOG is badly out of date.** Its `0.8.24` entry was written early in the cycle (by ticket 432) and only lists #239, #240, the memmap2 advisory, and 433. The entire second wave merged *after* that and is missing — plus there's an orphaned `## Unreleased` section above the dated entry that must fold into 0.8.24 (0.8.24 isn't tagged yet). 2. **No end-to-end confirmation the shipped fixes actually work in the release binary.** ~13 tickets shipped; we should smoke-test each one's repro against the built binary before tagging, to catch anything that silently regressed or only partially landed (436 shipped partial earlier in the cycle — exactly the kind of thing a smoke catches).

Imported from March ticket 445. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/445-release-readiness-v0-8-24-changelog-current-gate-version-confirmation-release-smoke-of-all-shipped-fixes
