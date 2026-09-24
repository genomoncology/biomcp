# Keep icacls output off the stdio MCP stream

Source issue: `sdlc/issues/2026-09-24-windows-icacls-output-corrupts-the-stdio-mcp-stream.md` (GitHub #283). Queue after the open 0.9.1 tickets.

## Problem

`src/cache/private.rs:492` runs `icacls.exe` with inherited handles. On Windows its localized success line lands in the stdio JSON-RPC stream on every managed write, and strict MCP clients break. The server's stdout must carry protocol frames only.

## Design

1. Run `icacls.exe` with `stdin`, `stdout`, and `stderr` all set to `Stdio::null()`, or capture stderr with `.output()` and put a trimmed, lossy copy of it in the "cannot secure" error. Inherited stdin also shares the protocol pipe.
2. Add a mechanical guard: a repository test that fails when non-test code under `src/` starts a child process without setting stdout explicitly. Allow-list `src/main_biomcp_cli.rs`, which is a pass-through launcher and must inherit.
3. Add a Windows stdio contract to the `windows-contracts` CI job. Start `biomcp serve`, initialize, send one `tools/call` that writes into a fresh managed content directory, and assert that every stdout line parses as JSON-RPC. The test must fail on the current code.
4. Keep the shell-out. Replacing it with `SetNamedSecurityInfo` through the `windows` crate removes the child process and the locale dependency, but it is a larger change. Record it as a follow-up issue if step 5 shows the per-call process count matters.
5. Measure the `icacls` runs per `search gwas` call. The reporter saw about 20. If the count comes from re-securing entries that are already secured, secure each entry once at creation and record the change.

## Acceptance

- The Windows contract fails before the fix and passes after it.
- The repository guard fails if the icacls `Stdio::null()` calls are removed.
- The CHANGELOG Unreleased section names ticket 1246 with a user-facing line.
- After release, a reply on #283 offers the reporter the patched wheel to test on zh-CN Windows. The reply needs Ian's OK.

## Review

- Design review: pending
- Code review: pending
