---
base: ae54cb7d3874a3793dd48883866f95850e74dc96
head: f8ef6216517316bbaae1d2f727729a693ecfbdf0
---
Architect ticket 329 decided benchmark stays an internal regression harness, not a public CLI. The code currently expresses that intent only through `#[cfg(test)] #[allow(dead_code)] mod benchmark;` in `src/cli/mod.rs:5-7`, and the top-level enum is named `BenchmarkCommand` — exactly the same shape a publicly-shipped Clap subcommand would have. A future contributor reading `src/cli/benchmark/mod.rs` cannot tell the harness from a public CLI surface without cross-referencing the architecture doc.

Imported from March ticket 343. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/343-mark-src-cli-benchmark-as-internal-regression-harness-in-code
