---
flow: build
priority: 5
---
# Decompose residual over-cap src/cli files under global ratchet

Ticket 334 adds a global 700-line cap ratchet for tracked Rust files under `src/cli`. The bootstrap allowlist keeps the ratchet green for the current residual files that already exceed the cap, but those entries must be removed by decomposition work rather than expanded.

Completed under March on 2026-04-29, as March ticket 347. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/347-decompose-residual-over-cap-src-cli-files-under-global-ratchet
