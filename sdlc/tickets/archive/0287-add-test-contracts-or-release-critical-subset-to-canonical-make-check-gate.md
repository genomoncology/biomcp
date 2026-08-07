---
flow: build
priority: 8
---
# Add test-contracts (or release-critical subset) to canonical make check gate

`make check` passed on the v0.8.22 release-readiness tree while `make test-contracts` was red on 2 landing-copy assertions. The canonical local gate cannot be green while the public contract suite is red, or the next release will ship the same class of drift.

Completed under March on 2026-04-24, as March ticket 287. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/287-add-test-contracts-or-release-critical-subset-to-canonical-make-check-gate
