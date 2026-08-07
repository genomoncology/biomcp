---
base: d8e9ca7a788ad91ea2fd04d3b898a9addc423948
head: 0df57f5d532be6a8106d5750942908adae2691e1
---
`tests/benchmark_cli_structure.rs` proves the file layout, line caps, and module headers under `src/cli/benchmark/`, and the structure ratchet is currently green. It does NOT prove the benchmark command is reachable in production. As of the 327 review, `target/release/biomcp benchmark --help` exits with `error: unrecognized subcommand 'benchmark'`, and `src/cli/mod.rs` declares the module `#[cfg(test)]` only. The structure ratchet passed while the public CLI claim was false.

Imported from March ticket 335. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/335-add-benchmark-runtime-wiring-ratchet-so-structure-tests-also-prove-cli-reachability
