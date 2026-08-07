---
base: 24dde96cbc912fb6e10cd2101b0c1042d684fa95
head: 2473b1129bd78c412e65533123dea7d0c020196a
---
`make test-contracts` currently fails five release-surface assertions on main, blocking any claim that the post-v0.8.21 surface is review-clean. The 0.8.22 release cannot be cut until this gate is green. The failures span tracked local artifacts, missing validation-profile source-of-record, changelog ticket-reference drift, and diagnostic docs/source-count drift.

Imported from March ticket 264. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/264-restore-make-test-contracts-to-green-before-v0-8-22-cut
