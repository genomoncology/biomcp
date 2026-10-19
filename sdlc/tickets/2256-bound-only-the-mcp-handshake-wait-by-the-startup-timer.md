---
priority: 2
---

# 2256 — Bound only the MCP handshake wait by the startup timer

Status: OPEN.

Milestone: 0.9.4

Reviews: revision cf355f4, accept

Reviews: revision 414bc9f, accept

## Outcome

PM extended the outcome: after startup timeout and guidance, the failing stdio server must exit promptly with the same failure status even while the client keeps stdin open. A hung client must not retain a server process.
Changes: `CHANGELOG.md`, `sdlc/tickets/2251-report-publisher-elided-references-as-empty.md`, `sdlc/tickets/2252-make-variant-articles-markdown-match-its-json.md`, `sdlc/tickets/2253-document-full-text-access-groups-and-reason-codes.md`, `sdlc/tickets/2254-fix-article-help-lists-and-condition-names.md`, `sdlc/tickets/2256-bound-only-the-mcp-handshake-wait-by-the-startup-timer.md`, `sdlc/tickets/2257-route-every-test-cache-root-through-the-shared-guard.md`, `src/main.rs`, `src/mcp/shell.rs`, `src/mcp/shell/pre_session.rs`, `tests/test_mcp_stdio_guidance.py`.

The MCP server's five-second startup timer limits only the wait for a client's initialize request. A supported tool call dispatched before initialization runs under its own invocation budget and is never aborted with the initialize-handshake error.

## Evidence

Found by 2247's MCP coverage: `run_stdio` wraps `await_initialize`, including supported pre-initialization `tools/call` dispatch, in the five-second startup timer, so a longer article operation aborts with the handshake error despite its own budget. Failure evidence is retained in experiment 792.

## Keeps

A client that never initializes still times out at five seconds. Initialized sessions are unchanged.

## Proof

A stdio test sends a supported pre-initialization tool call whose replayed provider reply takes longer than five seconds and receives the tool result. A silent client receives the handshake guidance and the server exits with the failure status while stdin remains open.

## Implementation boundary

Restate the outcome as a cumulative startup wait budget: fast pre-initialization traffic must not reset the wait indefinitely, and supported modern tool dispatch must not consume that wait budget. Preserve the existing initialized session and silent-client behavior. Prove the delayed supported call outside-in with authored local provider replies and retain prior failure evidence.

## Progress

- 2026-10-19 started

## Startup failure cleanup boundary

The controlled silent-client test reproduced guidance without process exit because the runtime awaited its blocking stdin reader. Scope cleanup to fatal stdio-server errors; preserve healthy initialized sessions, tool-error replies, HTTP behavior and the rest of the CLI. The open-pipe process-exit assertion must pass without closing the test input as a workaround.

## Verification

Baseline local-provider proof reproduced aborted supported pre-initialization work while the initialized equivalent succeeded. The expanded job 2256/1792429988105-88633 passed all fifteen owning cases, including delayed supported calls, subsequent initialize/ping, ordinary tool errors, HTTP compatibility, and status-one process exit while silent or pinging clients kept stdin open. Formatting, Python lint, all-target Rust lint and existing quality checks passed. All five product blobs matched the retained tested patch before unchanged-main integration. No live provider or full routine gate ran.

Keep: experiments/801-biomcp-2256-startup-wait — retain original and expanded red/green results, scripts, patches and source hashes.

The accepted batch carries the unchanged approved PM briefs for the remaining startup and cache-policy work. Product changes retain their own source review and focused checks.
