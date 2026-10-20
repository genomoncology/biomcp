---
base: dc47a1c7
head: 2b73809d
---

Release candidate construction now runs the canonical all-feature proof and
records that proof in the candidate manifest. Promotion rejects a candidate
whose manifest does not contain the required successful result, so a routine
build can no longer stand in for the optional-feature release binary.

Focused manifest and release-workflow tests passed. The final all-feature
lint, AlphaGenome tests, optimized release build, artifact smoke checks, and
offline release specifications all passed in the complete release gate.
