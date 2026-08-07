---
flow: architect
priority: 9
---
# Decide and document benchmark CLI ownership: public command or internal harness

`src/cli/benchmark/` contains a clap command module (`BenchmarkCommand::{Run, SaveBaseline, ScoreSession}`) with help text and a dispatcher, but `src/cli/mod.rs` declares it `#[cfg(test)]` only and `src/cli/commands.rs` has no `Benchmark` variant. `target/release/biomcp benchmark --help` exits with `error: unrecognized subcommand 'benchmark'`. At the same time, `architecture/technical/cli-decomposition-2026.md` says the `biomcp benchmark ...` command grammar remains unchanged. The architecture and the binary contradict each other; no public docs advertise the command.

Completed under March on 2026-04-27, as March ticket 329. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/329-decide-and-document-benchmark-cli-ownership-public-command-or-internal-harness
