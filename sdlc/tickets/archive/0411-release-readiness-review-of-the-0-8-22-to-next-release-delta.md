---
flow: review
priority: 8
---
# Release-readiness review of the 0.8.22 to next-release delta

1. **Code quality.** Read the substantive landed changes; flag correctness risks, dead code, leaked experiment/demo artifacts, over-broad public surface, and inconsistencies with the architecture docs. 2. **Test pass + spec-driven coverage.** Confirm `make lint` / `make test` / `make spec` are green on HEAD. For each headline capability landed since 0.8.22, confirm a behavioral mustmatch spec exists; name the capabilities that shipped without one. 3. **Verification stress.** Assess whether the live `make verify` lane actually exercises the new capabilities (diagnostics, VAERS/CVX, figshare asset retrieval, JATS converter, article fulltext) under realistic inputs — not just smoke. Name gaps where a capability has no live stress check. 4. **CI/gate speed.** Time each gate (`make lint`, `make test`, offline `make spec`, `make verify`) on HEAD and record wall-clock. Identify the slowest tests/specs and concrete dedup/tightening targets. 5. **Changelog.** Reconcile all `v0.8.22..HEAD` commits into a complete, truthful **Unreleased** section (grouped New features / Fixes / Internal), ticket-referenced in the existing CHANGELOG style — drafted as a deliverable in the review artifact, ready for `publish` to land at the version bump.

Completed under March on 2026-06-10, as March ticket 411. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/411-release-readiness-review-of-the-0-8-22-to-next-release-delta

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
