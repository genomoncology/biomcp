# 2356 — Isolate release-counterpart Make fixtures

Status: OPEN.

Milestone: 0.9.4

Priority: 1

Changes: tests/test_release_check_counterparts.py, CHANGELOG.md

Reviews: revision 4148919eb9d5c5e4776bef85d96e7f89b5b0df50, accept

## Outcome

Use the existing shared Make-fixture environment for the overlooked synthetic release-counterpart cases. A caller's Make command-line PATH and inherited Make controls must not override a fixture's fake commands. Preserve the real outer CPU-affinity check, both exact affinity assertions and all production gate checks. This is test isolation only; do not change production Make behavior or add a second isolation helper.

## Evidence

- Fresh Linux qualification j4420 on 9f6f165d455cf6c313dd4bb0108f0b937c7d1a7b passed all 4298 routine Rust cases and 2018 Python cases, with 12 expected Python skips, then failed test_release_check_counterparts.py::test_stress_uses_one_cpu_available_to_the_job. Its inherited Make command-line PATH defeated the fixture's fake taskset; the real taskset could not find the fixture's tools/run-offline. All later release stages were not reached. The failed run remains failed.
- Read-only root-cause review found that this case retains os.environ rather than using the shared make_fixture_env introduced by 2351. Fix the missed copy, including adjacent synthetic cases in this file only if they share the cause. Raw failed proof is retained on Yellow qualification-2330/j4420, manifest SHA256 d1b1a2c32c22e2601477db1234a742aa7cadb51d3d8d5ac0d761bdd6dba31a40.
- The developer restated the outcome before editing: synthetic release-counterpart commands own their environment so inherited GNU Make command-line PATH cannot select host commands; the real outer taskset restriction and both exact affinity assertions remain. The adjacent nightly fake-command runner shares the same inherited-environment cause. Changes are confined to this Python module and its descriptive changelog entry.
- Unchanged-source Gator j4421 ran exactly two invocations: an outer Make command-line PATH reproduced the same missing fixture runner failure; environmental PATH with inherited Make controls removed passed the unchanged case. Its raw manifest is 02021e63ac2f0063b893168c9254df4d672f2941d66a41f1bbe8f33cd7ddf2b2 under Yellow qualification-2330/j4421. This confirms the trigger, not a production Make defect.
- Gator j4422 passed all 16 cases in the complete affected module and Ruff, exit zero. Actual executed source was published main 9f6f165d455cf6c313dd4bb0108f0b937c7d1a7b plus repair patch SHA256 b76c87a758c54d77966767d0c92f1596d78b718fb8d19322167638534be3f6cb. The regression supplies hostile parent Make PATH and controls, while preserving real outer CPU affinity and both exact expected child-affinity lines. This is focused proof, not the pending landing gate.

- Focused proof archive: Yellow `ticket-evidence/2356/biomcp-2356-proof.tar.gz`, 3975 bytes, SHA256 `8a55777d7c8c8f4cce76742c85adb49c2018b04f7680b7463b3354a5c283ff08`. Both transfer endpoints and all five inner files were verified.

- Direct M5 landing gate j4423 passed on candidate `b4256332d707955230091f8cb2824e8c0b4f6211`, tree `beb87cd4e38c7b79d7b12a5c075a5dd26b52dbdc`, exit zero in 916599ms. It passed 4298 Rust cases, 2012 Python cases with 19 expected skips, strict documentation and 73 specification cases, 11 golden cases and 7 static cases. Darwin provides no kernel network isolation. Yellow `release-evidence/0.9.4/landing-2356` retains the exact command, source tree, source diff, scheduler receipt and raw log; verified manifest SHA256 `c8b80502fb7d0eeaeff1fc19b50606a1166a12c287cc7862573df85200cdff3d`. The gate was not repeated after publication.

## Proof

- Restate the outcome, reproduce the inherited command-line PATH failure against current main and name the exact files and failing outside-in test before editing code.
- Make the existing shared fixture helper own this boundary. Add a meaningful regression for inherited Make controls while preserving both CPU-affinity assertions and missing-tool/product-error propagation.
- Run only the complete affected Python module and its lint through pm job. Get independent review, then one declared landing gate. No new provider requests, private gate wrappers, skipped checks or floor changes.
