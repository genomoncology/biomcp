---
priority: 2
---

# 2170 — Keep Windows cache-lock contention a wait, and prove it on Windows

Status: OPEN.

Milestone: 0.9.3

## Outcome

On Windows, a contended cache lock waits for its deadline instead of failing at once with os error 33, a Windows regression test proves the contention mapping on a Windows runner, and a provider warning caused by a local I/O failure names the I/O kind and OS error code.

- Evidence: GitHub issue 292. On v0.9.2, fs2 0.4 returns Windows `ERROR_LOCK_VIOLATION` (os error 33) for a contended `try_lock_*`, not `ErrorKind::WouldBlock`, so the retry arms in `src/cache/clear.rs`, `src/cache/migration.rs` and `src/sources/gencc/store.rs` never match. Federated `search article` sources then lose the race, show as unavailable, and the command can exit with the I/O error. The reporter reproduced this 10 of 10 times on Windows 11 and showed that main at 74dfc40 waits and succeeds. Main already carries the code fix: bbe6ed316 (2072) replaced fs2 with `src/utils/fs_lock.rs`, which maps std's `TryLockError::WouldBlock` to `ErrorKind::WouldBlock`, and std reports Windows lock violations as `WouldBlock`. No other call site locks a file outside `fs_lock`, except the blocking `lock_shared` at `src/cache/clear.rs:286`, which waits by design.
- Evidence for the warning: `BioMcpError::external_failure` in `src/error.rs` classifies every non-HTTP error, I/O included, as `internal`, with the message `internal failure`, so the warning hides the OS error that would have named this defect.
- Starts from: `src/utils/fs_lock.rs`, the `windows-contracts` job in `.github/workflows/ci.yml`, and `src/error.rs`.
- Keeps: the std lock mapping, the deadline behavior of every lock caller, URL and secret redaction in external failure messages, and the 512-byte message bound.
- Changes: claims `src/utils/fs_lock.rs` `tests/windows_lock_contract.rs` `.github/workflows/ci.yml` `src/error.rs` `CHANGELOG.md`. Add a regression test that opens one lock file through two handles, locks it through the first, and asserts that `fs_lock::try_lock_exclusive` and `fs_lock::try_lock_shared` on the second return `ErrorKind::WouldBlock`; it runs on every platform and in the `windows-contracts` job. Classify `BioMcpError::Io` as `local_io` and put the I/O kind and raw OS error code in the message, never a path.
- Proof: before and after on a Windows runner, through a manual dispatch of `ci.yml`'s Windows job, which Ian authorized for this ticket. Before: the regression test fails with os error 33 on a throwaway branch whose `fs_lock` delegates to fs2 0.4's `try_lock_*`. After: the same test passes on the ticket branch. Record both run links in the ticket. A unit test shows that an I/O error with raw OS error 33 projects as class `local_io` with `os error 33` in the message and no path.
- Defers: the release itself, which is 0.9.3. Reply on issue 292 only with Ian's approval.
