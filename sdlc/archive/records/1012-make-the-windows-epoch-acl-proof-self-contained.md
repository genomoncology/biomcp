---
flow: quickfix
priority: 9
---

# Make the Windows epoch ACL proof self-contained

Exact-head CI run 31934806314 compiled the Windows boundary and passed the ordinary cache-root and hard-link tests, but `windows_cache_epoch_files_are_user_only_and_reject_hard_links` failed while inspecting the repaired epoch files. Its inline PowerShell calls `Get-Acl`; on the hosted Windows runner, PowerShell found that command in `Microsoft.PowerShell.Security` but could not load the module. The assertion then operated on null and rejected an ACL it never read.

Keep production cache permissions, migration behavior, the CI workflow, and the test's security contract unchanged. In `tests/managed_state_permissions.rs`, retain the existing `powershell.exe -NoProfile -NonInteractive` process, set `$ErrorActionPreference = 'Stop'`, and replace `Get-Acl` with the self-contained .NET Framework API `[System.IO.File]::GetAccessControl(path, [System.Security.AccessControl.AccessControlSections]::Access)`. Do not import or autoload `Microsoft.PowerShell.Security`. Require a protected DACL (`AreAccessRulesProtected`), exactly one explicit allow rule for the current Windows SID with exactly `FileSystemRights::FullControl`, and no inherited rule, after deliberately broadening both files and exercising the real cached fast-path repair. Preserve the hard-link rejection and no-`fsutil` assertions.

The focused hosted Windows test must pass and must fail if an Everyone rule, inherited rule, wrong SID, deny rule, or rights weaker/stronger than the exact protected contract survives. Linux compilation/tests must remain unaffected. Focused local checks, `make lint`, `git diff --check`, and the exact-head `windows-contracts` GitHub job must pass.
