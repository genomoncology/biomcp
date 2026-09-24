# Fix the MCP tool schemas and argument errors

Split from ticket 1238. Source issue:
`sdlc/issues/2026-09-23-mcp-tool-schemas-and-argument-errors.md`.

## Problem

Three tools (`search`, `get`, `variant_erepo`) publish a root schema of
only `type` and `oneOf` with no top-level `properties`, which some model
providers are reported to reject outright, and which is exactly why the
conformance suite's tools/call case stays unverified: it builds
arguments from top-level `properties`, finds none, sends `{}`, and gets
"entity must be a string". Argument validation failures return as
JSON-RPC `-32602` protocol errors instead of tool results with
`isError: true`, so a model never sees the reason. A garbage
`tools/list` cursor returns the full list instead of `-32602`.

## Design

1. Add top-level `properties.entity` with the entity enum and
   `required: ["entity"]` to every tool whose root is a bare `oneOf`.
   This is additive: providers that tolerate `oneOf` keep working, and
   tools/call conformance becomes exercisable. Live verification
   against each provider stays a recorded residual; it is an outward,
   metered action that needs separate authorization.
2. Return argument validation failures from tools/call as a tool result
   with `isError: true` carrying the validation message, per the MCP
   input-validation guidance. Protocol-level `-32602` remains for
   request-shape errors outside tool arguments.
3. Reject an unknown `tools/list` cursor with `-32602`.
4. Extend the conformance-style tests: tools/call with a real entity
   argument succeeds; an invalid argument yields `isError: true` with
   the message in content; a garbage cursor yields `-32602`.

## Acceptance

- Every tool schema carries top-level `properties` and `required`.
- Invalid arguments come back as `isError: true` tool results.
- Unknown cursors return `-32602`.
- The new tests cover all three behaviors.

## Review

- Design review: pending
- Code review: pending
