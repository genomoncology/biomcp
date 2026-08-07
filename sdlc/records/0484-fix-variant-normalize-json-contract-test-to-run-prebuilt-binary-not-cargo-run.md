---
base: b99eec75b683fe2e2cba8c5a5ad3d113015aaa53
head: 3429a28fb39784db56188e92a70777aadc3cb480
---
`tests/test_variant_normalize_json_contract.py::test_variant_normalize_json_no_result_emits_parseable_non_empty_stdout` invokes the BioMCP CLI via `["cargo", "run", "--quiet", "--bin", "biomcp", "--", ...]` under a 60-second `subprocess.run(timeout=60)`. That timeout exists to catch a runtime hang in `variant normalize`, but `cargo run` defaults to the **debug** profile — a separate build from the `cargo build --release --locked` that the harness runs before pytest. On any machine whose `target/debug` is already warm (local dev, March worktrees) `cargo run` is instant, so the test passes; on a clean CI runner the debug profile is cold, so `cargo run` triggers a full debug recompile that exceeds the 60s cap and the process is killed (`returncode -9`, `subprocess.TimeoutExpired`). This fails the release workflow's `validate` pytest step (and is a latent failure for PR CI and any cold local run).

Imported from March ticket 484. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/484-fix-variant-normalize-json-contract-test-to-run-prebuilt-binary-not-cargo-run
