---
priority: 2
---

# 2170 — Keep Windows cache-lock contention a wait, and prove it on Windows

Status: OPEN.

Milestone: 0.9.3

Reviews: revision fe548b1, accept

## Outcome

On Windows, a contended cache lock waits for its deadline instead of failing at once with os error 33, a Windows regression test proves the contention mapping on a Windows runner, and a provider warning caused by a local I/O failure names the I/O kind and OS error code.

- Evidence: GitHub issue 292. On v0.9.2, fs2 0.4 returns Windows `ERROR_LOCK_VIOLATION` (os error 33) for a contended `try_lock_*`, not `ErrorKind::WouldBlock`, so the retry arms in `src/cache/clear.rs`, `src/cache/migration.rs` and `src/sources/gencc/store.rs` never match. Federated `search article` sources then lose the race, show as unavailable, and the command can exit with the I/O error. The reporter reproduced this 10 of 10 times on Windows 11 and showed that main at 74dfc40 waits and succeeds. Main already carries the code fix: bbe6ed316 (2072) replaced fs2 with `src/utils/fs_lock.rs`, which maps std's `TryLockError::WouldBlock` to `ErrorKind::WouldBlock`, and std reports Windows lock violations as `WouldBlock`. The blocking `lock()` calls at `src/cache/clear.rs:211`, `224`, `286` and `316` and `src/cache/migration.rs:137` wait correctly on Windows.
- Evidence for the warning: `BioMcpError::external_failure` in `src/error.rs` classifies every non-HTTP error, I/O included, as `internal`, with the message `internal failure`, so the warning hides the OS error that would have named this defect.
- Starts from: `src/utils/fs_lock.rs`, the `windows-contracts` job in `.github/workflows/ci.yml`, and `src/error.rs`.
- Keeps: the std lock mapping, the deadline behavior of every lock caller, URL and secret redaction in external failure messages, and the 512-byte message bound.
- Changes: claims `tests/test_ci_workflow_contract.py` `src/utils/fs_lock.rs` `tests/windows_lock_contract.rs` `.github/workflows/ci.yml` `src/error.rs` `src/entities/article/backends.rs` `CHANGELOG.md`. Add a regression test that opens one lock file through two handles, locks it through the first, and asserts that `fs_lock::try_lock_exclusive` and `fs_lock::try_lock_shared` on the second return `ErrorKind::WouldBlock`; it runs on every platform and in the `windows-contracts` job, together with the existing contention tests `body_limit_epoch_lock_contention_obeys_variant_article_deadline` (`src/cache/migration.rs:692`) and `construction_under_a_held_epoch_lock_honors_the_deadline` (`src/entities/article/search/tests/deadline.rs:418`), which would have caught issue 292 but never run on Windows. Classify `BioMcpError::Io` as `local_io` and put the I/O kind and raw OS error code in the message, never a path. Log the Semantic Scholar error that `src/entities/article/backends.rs:754` drops.
- Proof: before and after on a Windows runner, through a manual dispatch of `ci.yml`'s Windows job, which Ian authorized for this ticket. Before: the regression test fails with os error 33 on a throwaway branch whose `fs_lock` delegates to fs2 0.4's `try_lock_*`. After: the same test passes on the ticket branch. Record both run links in the ticket. A unit test shows that an I/O error with raw OS error 33 projects as class `local_io` with `os error 33` in the message and no path.
- Defers: the release itself, which is 0.9.3. Ian approved the first reply on issue 292; the release update follows when 0.9.3 ships.

## Progress

- 2026-10-17 started

## Pre-code outcome and reproduction plan

At 27e59b197, preserve the existing standard lock/deadline behavior, prove its contention mapping on Windows, and project local I/O warnings as local_io with kind and OS code while redacting paths. Source inspection shows external_failure currently maps I/O to generic internal failure, backend error handling drops the Semantic Scholar error, and the Windows job omits the required deadline cases and new lock regression. This is an inspection result, not an executed red test. Use the authorized test-only fs2 wrapper branch for the Windows negative proof. For the projection red and green checks, apply test-only then implementation patches in one build job on the published base, then verify the published final revision; this carries no work-in-progress push. Extend the existing workflow contract for a narrow windows_only manual input. No release publication or issue reply is authorized.

## Proof and review

The Windows negative [run 39047221324](https://github.com/genomoncology/biomcp/actions/runs/39047221324) at a47d120d6 uses the test-only fs2 wrapper and fails all three contention/deadline cases. The positive [run 39047592032](https://github.com/genomoncology/biomcp/actions/runs/39047592032) at 37d0f628e explicitly passes all three cases. That revision's Rust is unchanged in the final reviewed source fe548b1e2. Review fixed Windows command failure propagation by selecting Bash; the negative workflow contract failed three cases in build job 1792253434943-56466 and all 24 passed with Python lint in 1792253448831-57351. The unchanged Rust projection and lock checks passed in 1792252886547-24631; lint passed in 1792252766760-2785. The Windows positive predates only the workflow shell correction and record merges. Final merged release qualification remains required.
