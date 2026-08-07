---
flow: build
priority: 8
---
# Decompose suggest.rs into src/cli/suggest/ submodules

`src/cli/suggest.rs` is 1,654 lines and currently mixes the route table, 15 route matcher functions, entity/anchor extraction helpers, a large `OnceLock` regex catalog, and inline tests. `biomcp suggest` is an agent-facing first-move surface, so the command must stay behavior-identical while the implementation is split into ownership zones that can evolve independently.

Completed under March on 2026-04-26, as March ticket 321. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/321-decompose-suggest-rs-into-src-cli-suggest-submodules
