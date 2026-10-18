---
priority: 2
---

# 2218 — Reconcile the byte total after explicit cache cleanup

Status: OPEN.

Milestone: 0.9.4
Depends on: 2202

## Outcome

Successful explicit cache cleanup reconciles the saved HTTP-content byte total with the content that remains, including after a failed full-filesystem write has legitimately left a conservative overcount. Preserve fail-soft provider answers, bounded maintenance and cross-process lock correctness.

- Evidence: reviewed2202 actual32MiB filesystem experiment on immutable9593a032b. Corrected build job `2202/1792350179375-53977` forced real errno28; provider answers survived with exit0 and no staging leak. After releasing filler, explicit `cache clean --max-size 1B` returned0/errors[] and left zero physical content but `.content-bytes-v1` still contained `v2 6295845`. Raw reproduction and disposable script are in owned experiment774 on the build host; the2202 record preserves exact command and artifact identity. Startup/automatic maintenance reconcile, but CLI explicit cleanup currently bypasses reconciliation.
- Changes: `src/cli/cache.rs` `src/cache/clean.rs` and the existing byte-total maintenance helper only as required, plus an outside-in CLI cache test and `CHANGELOG.md`. Declare exact final test/helper paths before editing. Reuse the shared accounting/locking path rather than duplicating arithmetic or inventing another marker.
- Keeps: conservative overcount before maintenance is allowed. No guarantee of provider-answer loss or size-limit breach is asserted by the review. No root filesystem filling, production cache mutation, or new cleanup framework.
- Proof: add an owning real-CLI regression that seeds valid cached content plus a legitimate conservative overcount, calls explicit cleanup, and fails when reported successful maintenance leaves a stale marker. Preserve exclusion with an active writer and timeout/fail-open behavior. Run targeted checks and rerun the existing bounded owned full-filesystem reproduction once after the fix; actual ENOSPC still returns the provider answer and successful cleanup reconciles the total. No broad repeated ENOSPC suite. Fresh review and normal landing/main checks follow.

## Progress

- 2026-10-18 started
