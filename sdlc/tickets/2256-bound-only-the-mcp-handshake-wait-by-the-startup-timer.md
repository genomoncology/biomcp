---
priority: 2
---

# 2256 — Bound only the MCP handshake wait by the startup timer

Status: OPEN.

Milestone: 0.9.4

## Outcome

Changes: `src/mcp/shell.rs`, `src/mcp/shell/pre_session.rs`, `tests/test_mcp_stdio_guidance.py`, `CHANGELOG.md`, `sdlc/tickets/2256-bound-only-the-mcp-handshake-wait-by-the-startup-timer.md`.

The MCP server's five-second startup timer limits only the wait for a client's initialize request. A supported tool call dispatched before initialization runs under its own invocation budget and is never aborted with the initialize-handshake error.

## Evidence

Found by 2247's MCP coverage: `run_stdio` wraps `await_initialize`, including supported pre-initialization `tools/call` dispatch, in the five-second startup timer, so a longer article operation aborts with the handshake error despite its own budget. Failure evidence is retained in experiment 792.

## Keeps

A client that never initializes still times out at five seconds. Initialized sessions are unchanged.

## Proof

A stdio test sends a supported pre-initialization tool call whose replayed provider reply takes longer than five seconds and receives the tool result. A silent client still gets the handshake timeout.

## Implementation boundary

Restate the outcome as a cumulative startup wait budget: fast pre-initialization traffic must not reset the wait indefinitely, and supported modern tool dispatch must not consume that wait budget. Preserve the existing initialized session and silent-client behavior. Prove the delayed supported call outside-in with authored local provider replies and retain prior failure evidence.
