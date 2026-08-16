---
flow: quickfix
priority: 9
---

# Pass the Windows ACL path outside command text

Exact-head CI run 31936545456 proved the delayed local fixture, both ordinary Windows managed-state tests, deliberate ACL broadening, and the repaired cache probe. It then failed before inspecting the ACL because `powershell.exe -Command` consumes the remaining command-line tokens as command text; the test appended the file path after the script and read `$args[0]`, which produced an invalid name for `[System.IO.File]::GetAccessControl`.

Keep production code, the local MyGene fixture, broadening, repair, hard-link rejection, and no-`fsutil` assertions unchanged. Change only `tests/managed_state_permissions.rs`. For each ACL inspection process, place the exact file path in a uniquely named process environment variable such as `BIOMCP_TEST_ACL_PATH`. Invoke `powershell.exe -NoProfile -NonInteractive -Command <script>` with the script as the final argument. The script must read the process environment variable explicitly and fail closed if it is missing or blank, not rooted, does not exist, or is not a regular file.

Continue using `[System.IO.File]::GetAccessControl(path, [System.Security.AccessControl.AccessControlSections]::Access)` with `$ErrorActionPreference = 'Stop'`. Require a protected DACL, exactly one explicit non-inherited `Allow` entry for the current Windows SID, and exactly `FileSystemRights::FullControl`. Capture PowerShell output and include its stderr in any Rust assertion failure. Add a focused Windows fail-closed check proving that a missing variable and a nonexistent path are rejected; then require both repaired epoch files to pass the exact contract. The focused hosted Windows test, local compile/checks, `make lint`, and `git diff --check` must pass.
