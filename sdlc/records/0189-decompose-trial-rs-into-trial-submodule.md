---
base: 8343d0bd09e3396e9595f4aba994d6f62cb502c2
head: 0209ea6c51d2b04417ff11b993fb3bf0cf4cdcb9
---
`src/entities/trial.rs` is 3,622 lines — the largest remaining entity file after the article, drug, and disease decompositions. Trial is a hot surface: NCI trial search contract alignment and terminated-status mapping both touched this file recently, and any future trial work will hit it. Shrinking it into a `src/entities/trial/` submodule following the same pattern as `src/entities/article/` makes future trial tickets scoped and fast.

Imported from March ticket 189. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/189-decompose-trial-rs-into-trial-submodule
