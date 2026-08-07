---
base: 72a67f552619ad901d1a066e9d72c73483bc02ba
head: 53bb4b7d32c591ffb4f5c0e3d8e5682b4335d541
---
Ticket 334 adds a global 700-line cap ratchet for tracked Rust files under `src/cli`. The bootstrap allowlist keeps the ratchet green for the current residual files that already exceed the cap, but those entries must be removed by decomposition work rather than expanded.

Imported from March ticket 347. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/347-decompose-residual-over-cap-src-cli-files-under-global-ratchet
