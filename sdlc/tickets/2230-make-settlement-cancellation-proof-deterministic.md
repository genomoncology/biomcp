# 2230 — Make settlement cancellation proof deterministic

Status: OPEN.

Milestone: 0.9.4

Priority: 1

Reviews: revision 64eae07, accept

Reviews: revision 9b28ebb, accept

## Outcome

Cancellation must stop an abandoned article provider retry and release admission. Diagnose the failed shared candidate before changing behavior or assertions. Preserve its observable cancellation contract and remove the actual race with the smallest fix.

- Changes: `src/cli/system/batch.rs` and `CHANGELOG.md`.
- Proof: The shared 2226 and 2111 candidate 3c13c0ae6f2041aecf4a568f39d0cd87a4fdba68 failed `cli::system::batch::tests::dropping_settlement_cancels_cacheable_article_provider_retry_and_admission` at batch.rs:807, left 2 versus right 1. The previous actual main 9e9bc75b5acabf38ae9852119368bc29efcb9602 passed its full gate. Establish whether the candidate caused the failure or an inherited timing race did. Reproduce against current main through the build job, record the exact command and result, then prove the fix with the same cancellation assertions. Do not add sleeps or loosen counts to hide a retry after cancellation. If more paths are needed, stop for a declared claim before edits.

## Baseline

The failed candidate gate is retained under gate-runs/biomcp/2226/m5/3c13c0ae6f2041aecf4a568f39d0cd87a4fdba68. No candidate source has reached main. The paper-anchor and Python consumer changes do not claim the failing batch implementation. A bounded repair lane owns investigation; no broad audit is authorized.

## Progress

- 2026-10-19 started
- 2026-10-19 landed 47f5577e783fc98a5b8b905b4a41f8db7c088590; next: Complete the accepted cancellation repair using its shared source gate.

## Cause and focused proof

The unchanged main cancellation test waits for real TCP readiness under a paused Tokio clock. With no runnable work, automatic time advancement can trigger a timeout and retry before readiness arrives. Candidate and main have identical cancellation, client, middleware and dependency inputs. This is an inherited test synchronization race.

On main 9e9bc75b5acabf38ae9852119368bc29efcb9602, build job `2230/1792413571787-94908` passed 40 focused iterations, then `2230/1792413668937-21826` reproduced the exact two-versus-one assertion at iteration 75. Command: `cargo nextest run --no-default-features -E 'test(dropping_settlement_cancels_cacheable_article_provider_retry_and_admission)' --stress-count 1000`.

Repair source ea5f2e16a2a3b755c7e9e6666703b97f84d494e3 keeps the runtime runnable while waiting and prioritizes the response notification. Existing cancellation, admission, request and cache assertions remain unchanged, with no sleeps or relaxed counts. Job `2230/1792413715803-24346` ran the same 1000-iteration command and make lint successfully, using published brief 0b343740b086371d7ef248b349e02a6078b88762 plus exact private patch SHA256 6bfbb79227b04156e526ed2f48b9975859aabe3e13f4f35a169702f1290cd93f. Final batch.rs SHA256 a791405b72f5b7062127d572dfbfa38e89624e35a7b56671007a23d23056d792 matches the tested file. The source commit includes the coordinator progress record; temporary patch files were removed. Fresh review and merged-candidate qualification remain required.
