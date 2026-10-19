# 2230 — Make settlement cancellation proof deterministic

Status: OPEN.

Milestone: 0.9.4

Priority: 1

## Outcome

Cancellation must stop an abandoned article provider retry and release admission. Diagnose the failed shared candidate before changing behavior or assertions. Preserve its observable cancellation contract and remove the actual race with the smallest fix.

- Changes: `src/cli/system/batch.rs` and `CHANGELOG.md`.
- Proof: The shared 2226 and 2111 candidate 3c13c0ae6f2041aecf4a568f39d0cd87a4fdba68 failed `cli::system::batch::tests::dropping_settlement_cancels_cacheable_article_provider_retry_and_admission` at batch.rs:807, left 2 versus right 1. The previous actual main 9e9bc75b5acabf38ae9852119368bc29efcb9602 passed its full gate. Establish whether the candidate caused the failure or an inherited timing race did. Reproduce against current main through the build job, record the exact command and result, then prove the fix with the same cancellation assertions. Do not add sleeps or loosen counts to hide a retry after cancellation. If more paths are needed, stop for a declared claim before edits.

## Baseline

The failed candidate gate is retained under gate-runs/biomcp/2226/m5/3c13c0ae6f2041aecf4a568f39d0cd87a4fdba68. No candidate source has reached main. The paper-anchor and Python consumer changes do not claim the failing batch implementation. A bounded repair lane owns investigation; no broad audit is authorized.
