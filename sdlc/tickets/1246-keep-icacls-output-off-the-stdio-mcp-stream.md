# Keep icacls output off the stdio MCP stream

Source issue: `sdlc/issues/2026-09-24-windows-icacls-output-corrupts-the-stdio-mcp-stream.md` (GitHub #283). Queue after the open 0.9.1 tickets.

## Problem

`src/cache/private.rs:492` runs `icacls.exe` with inherited handles. On Windows its localized success line lands in the stdio JSON-RPC stream on every managed write, and strict MCP clients break. The server's stdout must carry protocol frames only.

## Design

1. Run `icacls.exe` with `stdin`, `stdout`, and `stderr` all set to `Stdio::null()`, or capture stderr with `.output()` and put a trimmed, lossy copy of it in the "cannot secure" error. Inherited stdin also shares the protocol pipe.
2. Add a mechanical guard: a repository test that fails when non-test code under `src/` spawns a child whose stdout is unset or `Stdio::inherit()`. Only `Stdio::null()`, `Stdio::piped()`, or `.output()` pass. Allow-list exactly `src/main_biomcp_cli.rs`, the pass-through launcher that must inherit. "Non-test" means code before the first `#[cfg(test)]` in a file, plus the separate-test-module shapes `src/**/tests/**` and `*tests.rs`, which the guard skips (matching the existing source-scan pattern in `tests/test_source_package_boundary.py`).
3. Add a Windows stdio contract to the `windows-contracts` CI job, driven offline: spawn `env!("CARGO_BIN_EXE_biomcp")` (the pattern `tests/managed_state_permissions.rs` uses) with a fresh `BIOMCP_CACHE_DIR` and `BIOMCP_MYGENE_BASE` pointed at a refused local port, initialize, then send one HTTP-backed `tools/call` (a gene search). Shared-client construction secures the cache tree before the request completes (`src/sources/mod.rs:943,960`), so the call triggers the icacls path even though the fetch fails, and the failure returns an isError tool result. Assert every stdout line parses as JSON-RPC. The test must fail on the current code.
4. Keep the shell-out. The in-process alternative is modest, not large: `src/cache/private.rs:255-330` already uses `SetSecurityInfo` for current-user-only repair of managed files, and extending it to directories (OI/CI inheritance) would remove the child process and the locale dependency. Record it as a follow-up issue if memoization in step 5 does not bring the steady-state count to zero.
5. Measure the `icacls` runs per `search gwas` call. The reporter saw about 20; today every write re-secures every touched path (`src/cache/private.rs:92-131`). Fix by per-process memoization of secured paths: the first encounter in a process still repairs pre-existing or externally broadened directories (the re-securing posture `write_security_windows_tests.rs:75-101` encodes stays true), and steady-state writes spawn nothing. Record the measured before/after.

## Acceptance

- The Windows contract fails before the fix and passes after it.
- The repository guard fails if the icacls `Stdio::null()` calls (or the capture branch) are removed, and passes on the rest of the tree today.
- The CHANGELOG Unreleased section names ticket 1246 with a user-facing line.
- Ian approved a thank-you on #283, posted 2026-09-24 (https://github.com/genomoncology/biomcp/issues/283#issuecomment-5816964998). It promises the fix in the next release and accepts the reporter's offer to test on zh-CN Windows. When the patched wheel exists, point the reporter to it; that follow-up reply needs Ian's OK.

## Review

- Design review: REJECT once (guard trigger contradicted its
  allow-list; the contract test had no offline mechanism; step 5
  dropped the re-securing posture), findings folded, re-review ACCEPT
  2026-09-24
- Code review: pending
- Implementation note: no Windows host exists for local measurement;
  the "before" count is the reporter's ~20 spawns per call, and the
  memo makes steady state zero by construction (every path after its
  first secure returns from the memo). The windows-contracts job now
  runs the stdio contract and the `cache::private` unit tests on real
  Windows, which is where both are falsifiable.
