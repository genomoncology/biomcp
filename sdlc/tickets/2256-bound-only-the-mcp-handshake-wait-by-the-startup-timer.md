---
priority: 2
---

# 2256 — Bound only the MCP handshake wait by the startup timer

Status: OPEN.

Milestone: 0.9.4

## Outcome

The MCP server's five-second startup timer limits only the wait for a client's initialize request. A supported tool call dispatched before initialization runs under its own invocation budget and is never aborted with the initialize-handshake error.

## Evidence

Found by 2247's MCP coverage: `run_stdio` wraps `await_initialize`, including supported pre-initialization `tools/call` dispatch, in the five-second startup timer, so a longer article operation aborts with the handshake error despite its own budget. Failure evidence is retained in experiment 792.

## Keeps

A client that never initializes still times out at five seconds. Initialized sessions are unchanged.

## Proof

A stdio test sends a supported pre-initialization tool call whose replayed provider reply takes longer than five seconds and receives the tool result. A silent client still gets the handshake timeout.
