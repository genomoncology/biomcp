# 2358 — Restore atomic Windows skill repair

Status: OPEN.

Milestone: 0.9.4

Priority: 1

Changes: src/cli/skill/atomic_swap.rs, src/cli/skill/install.rs, docs/getting-started/skills.md

## Outcome

Restore the published forced skill-repair contract on Windows while preserving the existing all-managed-files publication, unrelated-file preservation, failure cleanup and concurrency guarantees. Use supported documented platform operations. Do not replace atomic publication with copy/delete, alter the public layout or weaken the installed-wheel proof.

## Evidence

- The Windows native rehearsal failure exposed missing Windows publication support. Fresh installation is separately tracked in2357. Read-only inspection also found unconditional rejection of Windows in the force-repair directory-exchange path. Documentation promises forced repair without a Windows exception, and no related ruling permits removing it.
- Before implementation, determine from existing repository contracts and official platform documentation whether a safe platform implementation preserves the approved guarantees. A weaker transaction or changed storage layout requires a product decision. Any such decision is deferred until tomorrow under the user's instruction; send no notifications.
- Bounded platform review found no documented supported directory-exchange substitute. MoveFileExW rejects replacement of an existing directory; FILE_RENAME_INFORMATION does not specify exchange; ReplaceFileW replaces a single file. Moving the old directory aside creates a missing-target interval and crash-recovery gap. Transactional NTFS introduces a deprecated platform dependency and does not by documentation alone prove this directory sequence. Fresh publication remains independently repairable in2357.

PM decision needed: Windows has no established supported directory-exchange implementation for the current skill layout. Choose an explicit Windows forced-repair limitation, approve a versioned-layout design, or accept a Transactional NTFS dependency and its platform restrictions. Preserve the existing fail-closed behavior until that decision. This outside input is deferred until tomorrow; no notification was sent.

The bounded review recommends an explicit near-term Windows forced-repair limitation, requiring a small documentation/help change and native refusal/preservation proof. A versioned layout needs migration, reader compatibility, cleanup and concurrency design. Transactional NTFS adds availability and filesystem restrictions and is not recommended. Renaming the old directory aside violates the current visibility guarantee and is excluded from this outcome. This is a recommendation, not an approved change to the contract.

Platform sources: [MoveFileExW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw), [FILE_RENAME_INFORMATION](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_rename_information), [ReplaceFileW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew), [Microsoft's Transactional NTFS recommendation](https://learn.microsoft.com/en-us/windows/win32/fileio/deprecation-of-txf), [transaction restrictions](https://learn.microsoft.com/en-us/windows/win32/fileio/programming-considerations-for-transacted-fileio-), and [MoveFileTransactedW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefiletransactedw).

## PM ruling

PM ruling 2026-10-22: take the explicit Windows forced-repair limitation for 0.9.4. Released 0.9.3 already refuses forced repair on Windows (the platform guard dates from 2026-07-21), so this documents existing behavior and is no regression. Keep the fail-closed refusal. Make the refusal message tell the user how to repair on Windows: remove the managed skill folder, then run a fresh install. State the limitation in docs/getting-started/skills.md and the skill install help. Prove at the native Windows executable that forced repair refuses, leaves the existing bundle and unrelated files untouched, and that the stated manual repair succeeds. A versioned layout that would allow atomic Windows repair is a later design, outside 0.9.4. Transactional NTFS and rename-aside stay excluded.

## Proof

- Reproduce forced repair with the actual native Windows executable or record exactly why its existing unconditional platform guard already establishes the failure, without relabeling fresh-install evidence as force execution.
- Establish old-or-new bundle visibility, preservation of unrelated files, rollback/failure behavior and two-process ownership at the actual executable boundary. A fresh-install fix alone does not satisfy this ticket.
- Record a line beginning PM decision needed: if the approved guarantees cannot be restored with a bounded supported implementation. Keep this ticket open and defer that outside input until tomorrow.
