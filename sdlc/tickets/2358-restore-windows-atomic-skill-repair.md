# 2358 — Document Windows manual skill repair

Status: COMPLETE.
Landed: 3a72c13f53cf25b0978a68fe03681dcfd89366b0

Milestone: 0.9.4

Priority: 1

Changes: src/cli/skill/atomic_swap.rs, src/cli/skill/mod.rs, docs/getting-started/skills.md, tests/skill_install_contract.rs, tests/test_public_skill_docs_contract.py, CHANGELOG.md, sdlc/records/2358-windows-manual-skill-repair.md

Reviews: revision e6abc1752e3889279d7e8c70b590b035beb2a08c, accept

Reviews: revision 3913ab145137542f15fa44072e24be34173163a2, accept

## Outcome

Document the existing Windows forced-repair limitation for 0.9.4 and preserve its refusal without changing the existing managed bundle or unrelated files. Tell users in the refusal message, skill install help and skills page to back up unrelated files outside the managed skill folder, remove that folder, run a fresh install and restore the saved unrelated files. Prove refusal preservation and successful manual repair through the native Windows executable. Keep the current layout and fresh-install publication guarantees. Transactional NTFS, rename-aside and a versioned layout remain outside this change.

## Evidence

- The Windows native rehearsal failure exposed missing Windows publication support. Fresh installation is separately tracked in2357. Read-only inspection also found unconditional rejection of Windows in the force-repair directory-exchange path. Documentation previously promised forced repair without a Windows exception. The PM ruling below now authorizes an explicit Windows limitation.
- The bounded assessment reviewed repository contracts and official platform documentation. Weaker publication semantics and a changed layout required a product decision; the PM ruling below chooses the limitation instead.
- Bounded platform review found no documented supported directory-exchange substitute. MoveFileExW rejects replacement of an existing directory; FILE_RENAME_INFORMATION does not specify exchange; ReplaceFileW replaces a single file. Moving the old directory aside creates a missing-target interval and crash-recovery gap. Transactional NTFS introduces a deprecated platform dependency and does not by documentation alone prove this directory sequence. Fresh publication remains independently repairable in2357.

The PM ruling below supersedes the earlier implementation assessment and resolves its request for a product decision. The bounded assessment found no supported directory-exchange substitute for the current layout. Windows forced repair remains refused; the authorized work documents that limitation and supplies a manual repair procedure that preserves unrelated files through backup and restoration.

Platform sources: [MoveFileExW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw), [FILE_RENAME_INFORMATION](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_rename_information), [ReplaceFileW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew), [Microsoft's Transactional NTFS recommendation](https://learn.microsoft.com/en-us/windows/win32/fileio/deprecation-of-txf), [transaction restrictions](https://learn.microsoft.com/en-us/windows/win32/fileio/programming-considerations-for-transacted-fileio-), and [MoveFileTransactedW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefiletransactedw).

## PM ruling

PM ruling 2026-10-22: take the explicit Windows forced-repair limitation for 0.9.4. Released 0.9.3 already refuses forced repair on Windows (the platform guard dates from 2026-07-21), so this documents existing behavior and is no regression. Keep the fail-closed refusal. Make the refusal message tell the user how to repair on Windows: remove the managed skill folder, then run a fresh install. State the limitation in docs/getting-started/skills.md and the skill install help. Prove at the native Windows executable that forced repair refuses, leaves the existing bundle and unrelated files untouched, and that the stated manual repair succeeds. A versioned layout that would allow atomic Windows repair is a later design, outside 0.9.4. Transactional NTFS and rename-aside stay excluded.

## Proof

- Start from the PM ruling on current main and reproduce the missing Windows limitation in the actual install help. The existing platform guard establishes forced refusal; do not relabel fresh-install evidence as native force execution.
- Keep an executable Windows case that installs a managed bundle, changes its guide, adds unrelated files, invokes forced repair, checks the refusal instructions, compares every existing file before and after refusal and checks abandoned staging cleanup.
- Follow the stated manual procedure at the executable boundary: back up unrelated files outside the managed folder, remove only that folder, reinstall without force and restore those files. Check current status, rendered-guide equality and preservation of unrelated files and neighboring skills.
- Run affected local checks through pm job and retain source-to-patch binding. Obtain fresh review, then run the existing native Windows qualification route through the coordinator. The executed proof and its limits are recorded in [Windows manual skill repair](../records/2358-windows-manual-skill-repair.md).
