# Windows icacls output corrupts the stdio MCP stream

Filed 2026-09-24 from GitHub #283. Confirmed by reading the code at bc13665a. Not reproduced on Windows here.

## Symptom

On Windows, `biomcp serve` over stdio writes lines like `Successfully processed 1 files; Failed processing 0 files` between JSON-RPC frames whenever a tool call writes into the managed content directory. The text is localized, so a zh-CN console emits GBK bytes. Strict clients drop the session. The reporter counted about 20 `icacls.exe` starts during one `search gwas` call and runs biomcp behind a shim that keeps only lines starting with `{`.

## Cause

`src/cache/private.rs:492` runs `icacls.exe` with `.status()`. The child inherits biomcp's stdin, stdout, and stderr, and in stdio mode that stdout is the protocol stream. `secure_entry` runs on every managed write, and with `recurse` it runs once per directory entry, which explains the process count.

The `windows-contracts` CI job only runs `cargo check` and the `managed_state_permissions` test. Nothing checks that stdout stays clean in stdio mode on Windows.

## Fix

Ticket 1246.

## Resolved

Ticket 1246. The icacls child runs with all three streams nulled;
steady-state writes spawn nothing (per-process memoization, repair on
first encounter); a source guard rejects unset or inheriting stdout on
any new spawn in production code; a Windows stdio contract fails on
the old code and runs in CI. The reporter reply waits for the release
and Ian's OK. See
`sdlc/records/1246-keep-icacls-output-off-the-stdio-mcp-stream.md`.
