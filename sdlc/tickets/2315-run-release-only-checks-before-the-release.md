---
priority: 2
---

# 2315 — Run release-only checks before the release

Status: OPEN.

Milestone: 0.9.4

Depends on: 2316, 2319

## Outcome

`make nightly-checks` composes the existing stress, optimized panic-recovery, protobuf generation and long-suite checks, the repository audit, full-feature/rendered-output checks and installed-wheel smoke, followed by the feature-coverage target from 2319. One counterpart guard detects a release check without its actual local/nightly counterpart; native-platform and publication-only boundaries remain explicit.

The new pm scheduler owns recurring nightly execution after migration. This ticket adds no private timer or scheduler. Until that scheduler runs it, execute the release-only checks before each release through the full rehearsal. The complete Windows jobs in the final full rehearsal supply this milestone's Windows proof; run no separate CI workflow.

- Proof: The real nightly entrypoint invokes the required checks and preserves failure status. A planted release check without a counterpart fails the guard. The final complete release rehearsal supplies the release-only and Windows execution proof; do not claim a recurring nightly run before scheduler deployment.

## Evidence

0.9.4's first rehearsal found the fresh-root wheel bug and two smoke gaps that no earlier check ran; 2291 covered ci.yml but not release.yml. From Ian's process assessment of 2026-10-20.

PM rulings: m2502 confirms complete rehearsal Windows proof; m2505 assigns scheduling to the new pm scheduler and limits this implementation to the repo target and counterpart guard. m2513 puts the coverage floor in nightly/release checks.

PM ruling m2529 removes full main checks after each landing. The nightly target owns the extra checks; until scheduled, run main-gate once on the final candidate before rehearsal.
