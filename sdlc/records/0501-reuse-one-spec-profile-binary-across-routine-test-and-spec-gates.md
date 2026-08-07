---
base: 5daadc07777d9f8fc0235e90fd85b78fe9de3869
head: f86c87d41919ae8d2d8d2db519c7ed55a87fd930
---
BioMCP's routine full gate compiles three materially different Rust outputs: nextest test binaries, a thin-LTO `release` binary in `make test-contracts`, and then a separate `spec` binary in `make spec`. The release profile uses `lto = "thin"` and `codegen-units = 1`; it is intentionally expensive and unnecessary for routine Python CLI contracts. The Python tests already honor `BIOMCP_BIN`, and the `spec` profile is the repository's purpose-built fast executable-contract binary.

Imported from March ticket 501. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/501-reuse-one-spec-profile-binary-across-routine-test-and-spec-gates
