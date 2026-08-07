---
flow: build
priority: 5
---
# Decompose trial.rs into trial submodule

`src/entities/trial.rs` is 3,622 lines — the largest remaining entity file after the article, drug, and disease decompositions. Trial is a hot surface: NCI trial search contract alignment and terminated-status mapping both touched this file recently, and any future trial work will hit it. Shrinking it into a `src/entities/trial/` submodule following the same pattern as `src/entities/article/` makes future trial tickets scoped and fast.

Completed under March on 2026-04-14, as March ticket 189. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/189-decompose-trial-rs-into-trial-submodule
