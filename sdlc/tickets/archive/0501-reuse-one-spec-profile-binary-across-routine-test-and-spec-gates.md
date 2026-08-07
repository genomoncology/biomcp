---
flow: build
priority: 5
---
# Reuse one spec-profile binary across routine test and spec gates

BioMCP's routine full gate compiles three materially different Rust outputs: nextest test binaries, a thin-LTO `release` binary in `make test-contracts`, and then a separate `spec` binary in `make spec`. The release profile uses `lto = "thin"` and `codegen-units = 1`; it is intentionally expensive and unnecessary for routine Python CLI contracts. The Python tests already honor `BIOMCP_BIN`, and the `spec` profile is the repository's purpose-built fast executable-contract binary.

Completed under March on 2026-07-11, as March ticket 501. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/501-reuse-one-spec-profile-binary-across-routine-test-and-spec-gates
