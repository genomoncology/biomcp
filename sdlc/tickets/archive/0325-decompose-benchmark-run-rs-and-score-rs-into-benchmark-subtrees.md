---
flow: build
priority: 3
---
# Decompose benchmark run.rs and score.rs into benchmark subtrees

`src/cli/benchmark/run.rs` is 1,344 lines and `src/cli/benchmark/score.rs` is 824 lines. Together they keep the benchmark command family over the 700-line cap and mix suite configuration, subprocess execution, regression analysis, report formatting, command normalization, token/error extraction, and inline tests. The benchmark family already has its own namespace, so it can be decomposed without changing the public `biomcp benchmark ...` grammar.

Completed under March on 2026-04-27, as March ticket 325. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/325-decompose-benchmark-run-rs-and-score-rs-into-benchmark-subtrees
