---
flow: quickfix
priority: 7
---
# Add benchmark runtime-wiring ratchet so structure tests also prove CLI reachability

`tests/benchmark_cli_structure.rs` proves the file layout, line caps, and module headers under `src/cli/benchmark/`, and the structure ratchet is currently green. It does NOT prove the benchmark command is reachable in production. As of the 327 review, `target/release/biomcp benchmark --help` exits with `error: unrecognized subcommand 'benchmark'`, and `src/cli/mod.rs` declares the module `#[cfg(test)]` only. The structure ratchet passed while the public CLI claim was false.

Completed under March on 2026-04-28, as March ticket 335. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/335-add-benchmark-runtime-wiring-ratchet-so-structure-tests-also-prove-cli-reachability
