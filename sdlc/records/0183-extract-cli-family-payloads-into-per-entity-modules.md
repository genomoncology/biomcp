---
base: ef7bf8c62be39d3176f297a8c9c66814b710dcff
head: c2e4d5fab3bf2124a44309e78eb50c2ddad1a17f
---
The first pass at decomposing `src/cli/mod.rs` (12,780 lines) landed the shared scaffolding: `src/cli/types.rs` (shared types) and `src/cli/test_support.rs` (test helpers). The remaining 12,000+ lines are dominated by command payload structs, subcommand enums, runtime dispatch, and per-entity tests.

Imported from March ticket 183. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/183-extract-cli-family-payloads-into-per-entity-modules
