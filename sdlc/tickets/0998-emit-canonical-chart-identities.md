---
flow: quickfix
priority: 9
---

# Emit canonical chart identities

Structured chart output must use the exact public CLI spelling for every chart. In particular, `stacked-bar` must never become `stackedbar`. One canonical mapping should serve all twelve chart names.

Red-green coverage belongs in `src/cli/tests/facade/chart.rs`; its JSON identity assertions may be expanded or restated for the complete catalog.
