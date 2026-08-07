---
base: 80c4ca8aecf35ba33cc706875c5cd7eedf4fb397
head: c14856d36061c96ee152560367b027f9ea69ee22
---
`src/cli/benchmark/` contains a clap command module (`BenchmarkCommand::{Run, SaveBaseline, ScoreSession}`) with help text and a dispatcher, but `src/cli/mod.rs` declares it `#[cfg(test)]` only and `src/cli/commands.rs` has no `Benchmark` variant. `target/release/biomcp benchmark --help` exits with `error: unrecognized subcommand 'benchmark'`. At the same time, `architecture/technical/cli-decomposition-2026.md` says the `biomcp benchmark ...` command grammar remains unchanged. The architecture and the binary contradict each other; no public docs advertise the command.

Imported from March ticket 329. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/329-decide-and-document-benchmark-cli-ownership-public-command-or-internal-harness
