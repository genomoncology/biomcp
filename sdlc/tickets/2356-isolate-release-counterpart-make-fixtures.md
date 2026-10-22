# 2356 — Isolate release-counterpart Make fixtures

Status: OPEN.

Milestone: 0.9.4

Priority: 1

Changes: tests/test_release_check_counterparts.py, CHANGELOG.md

## Outcome

Use the existing shared Make-fixture environment for the overlooked synthetic release-counterpart cases. A caller's Make command-line PATH and inherited Make controls must not override a fixture's fake commands. Preserve the real outer CPU-affinity check, both exact affinity assertions and all production gate checks. This is test isolation only; do not change production Make behavior or add a second isolation helper.

## Evidence

- Fresh Linux qualification j4420 on 9f6f165d455cf6c313dd4bb0108f0b937c7d1a7b passed all 4298 routine Rust cases and 2018 Python cases, with 12 expected Python skips, then failed test_release_check_counterparts.py::test_stress_uses_one_cpu_available_to_the_job. Its inherited Make command-line PATH defeated the fixture's fake taskset; the real taskset could not find the fixture's tools/run-offline. All later release stages were not reached. The failed run remains failed.
- Read-only root-cause review found that this case retains os.environ rather than using the shared make_fixture_env introduced by 2351. Fix the missed copy, including adjacent synthetic cases in this file only if they share the cause. Raw failed proof is retained on Yellow qualification-2330/j4420, manifest SHA256 d1b1a2c32c22e2601477db1234a742aa7cadb51d3d8d5ac0d761bdd6dba31a40.

## Proof

- Restate the outcome, reproduce the inherited command-line PATH failure against current main and name the exact files and failing outside-in test before editing code.
- Make the existing shared fixture helper own this boundary. Add a meaningful regression for inherited Make controls while preserving both CPU-affinity assertions and missing-tool/product-error propagation.
- Run only the complete affected Python module and its lint through pm job. Get independent review, then one declared landing gate. No new provider requests, private gate wrappers, skipped checks or floor changes.
