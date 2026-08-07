---
flow: quickfix
priority: 7
---
# Mark src/cli/benchmark/ as internal regression harness in code

Architect ticket 329 decided benchmark stays an internal regression harness, not a public CLI. The code currently expresses that intent only through `#[cfg(test)] #[allow(dead_code)] mod benchmark;` in `src/cli/mod.rs:5-7`, and the top-level enum is named `BenchmarkCommand` — exactly the same shape a publicly-shipped Clap subcommand would have. A future contributor reading `src/cli/benchmark/mod.rs` cannot tell the harness from a public CLI surface without cross-referencing the architecture doc.

Completed under March on 2026-04-28, as March ticket 343. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/343-mark-src-cli-benchmark-as-internal-regression-harness-in-code
