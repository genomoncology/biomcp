---
flow: build
priority: 9
---
# Restore make test-contracts to green before v0.8.22 cut

`make test-contracts` currently fails five release-surface assertions on main, blocking any claim that the post-v0.8.21 surface is review-clean. The 0.8.22 release cannot be cut until this gate is green. The failures span tracked local artifacts, missing validation-profile source-of-record, changelog ticket-reference drift, and diagnostic docs/source-count drift.

Completed under March on 2026-04-21, as March ticket 264. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/264-restore-make-test-contracts-to-green-before-v0-8-22-cut
