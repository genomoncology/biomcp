---
priority: 3
---

# 2263 — Surface MCP argument parse errors to the model

Status: OPEN.

Milestone: 0.9.4

## Outcome

A malformed MCP tool call, such as `gene_cspec` without `gene`, returns an `isError` tool result whose message names the bad argument, so the calling model can correct itself. The `variant_erepo` root schema can no longer drift from its branches.

## Evidence

Residuals of 1251 in `sdlc/issues/2026-09-24-mcp-schema-follow-ups-after-1240.md`: parse failures inside rmcp's `Parameters` stay protocol errors (-32602) that the model never sees; and the `variant_erepo` flat root restates its property list by hand.

## Keeps

Valid calls are unchanged. Unknown-field rejection keeps working. Keep the recorded -32602 channel only where the spec requires a protocol error.

## Proof

A stdio test sends `gene_cspec` without `gene` and receives an `isError` result naming `gene`. A test derives the `variant_erepo` root property set from its branch types and asserts the published schema matches.
