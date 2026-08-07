---
flow: quickfix
priority: 5
---
# Fix variant-normalize JSON contract test to run prebuilt binary (not cargo run)

`tests/test_variant_normalize_json_contract.py::test_variant_normalize_json_no_result_emits_parseable_non_empty_stdout` invokes the BioMCP CLI via `["cargo", "run", "--quiet", "--bin", "biomcp", "--", ...]` under a 60-second `subprocess.run(timeout=60)`. That timeout exists to catch a runtime hang in `variant normalize`, but `cargo run` defaults to the **debug** profile — a separate build from the `cargo build --release --locked` that the harness runs before pytest. On any machine whose `target/debug` is already warm (local dev, March worktrees) `cargo run` is instant, so the test passes; on a clean CI runner the debug profile is cold, so `cargo run` triggers a full debug recompile that exceeds the 60s cap and the process is killed (`returncode -9`, `subprocess.TimeoutExpired`). This fails the release workflow's `validate` pytest step (and is a latent failure for PR CI and any cold local run).

Completed under March on 2026-07-08, as March ticket 484. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/484-fix-variant-normalize-json-contract-test-to-run-prebuilt-binary-not-cargo-run
