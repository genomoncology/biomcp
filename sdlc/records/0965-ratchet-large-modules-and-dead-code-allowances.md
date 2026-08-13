---
base: 0169d9e8e67321c24cc95df7034c72e22dcd312a
head: b6242f564d33c4688896d3b949d45f72eaec7752
---

Canonical quality checks now pin all 33 tracked Rust source files above 1,000
physical lines to exact baselines. New oversized files, growth, stale or renamed
entries, symlink escapes, and unexplained baseline increases fail. The explicit
update command lowers baselines automatically and accepts one increase only
with a named path, ticket, exact delta, reason, and removal condition.

All 127 non-generated dead-code allowances now have checked path/item identity,
an owner, their adjacent concrete reason, and a removal condition. Whole-file
and module allowances are visible as broad exceptions. Only the exact generated
AlphaGenome binding path is excluded; near matches remain checked.

Removed PubMed's three parallel request-plan projections, their three adapter
methods, the module-wide dead-code allowance, and their duplicate tests. PubMed
and LitSense2 construction tests now inspect the production `RequestPlan`.
Production `src/` shrank by 244 net lines.

All 53 quality-ratchet contract tests, both new canonical audits, 95 focused
PubMed/LitSense2 tests, and full no-default-feature Clippy passed.
