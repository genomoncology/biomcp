---
base: a7216faf6b3c1b96755d1acf43fd81934a1bfb29
head: ade40354421194c9a963047bf1715405188c3c67
---
`src/cli/benchmark/run.rs` is 1,344 lines and `src/cli/benchmark/score.rs` is 824 lines. Together they keep the benchmark command family over the 700-line cap and mix suite configuration, subprocess execution, regression analysis, report formatting, command normalization, token/error extraction, and inline tests. The benchmark family already has its own namespace, so it can be decomposed without changing the public `biomcp benchmark ...` grammar.

Imported from March ticket 325. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/325-decompose-benchmark-run-rs-and-score-rs-into-benchmark-subtrees
