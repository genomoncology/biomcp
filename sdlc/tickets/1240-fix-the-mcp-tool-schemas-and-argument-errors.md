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

Revised after the design review (two P1 scope corrections folded in;
the reviewer verified every mechanism against the code and the tracked
spec revision 2026-07-28, vendored by @hasmcp/mcp-spec-test@0.1.5).

1. Root schemas. `search` and `get` roots gain top-level
   `properties.entity` — an enum derived mechanically from the same
   branch lists the schemas already build (search's eight entities at
   `shell.rs:290-292`, get's thirteen `typed_get_capabilities()`) so
   the root enum cannot drift from the branch consts — plus
   `required: ["entity"]`, keeping the `oneOf` branches (AND
   semantics; no root `additionalProperties: false`, so branch fields
   stay legal). `variant_erepo` has no entity concept: its root gains
   top-level `properties` listing the union of selector fields (all
   optional) while keeping its `oneOf` branches. Acceptance restated:
   every tool root carries top-level `properties`; `search` and `get`
   additionally carry `required: ["entity"]`. rmcp is locked at 1.7.0
   (minimum 1.1.1).
2. Argument errors. Recorded decision, not a compliance claim: the
   tracked spec sanctions `-32602` for invalid tool arguments and says
   errors that originate from the tool SHOULD be reported in the
   result with `isError: true`. We move the in-body argument
   validations to `Ok(CallToolResult::error(...))` (isError with the
   message as text) because the spec's stated rationale — the model
   sees the failure and self-corrects — is the behavior we want.
   Protocol-shape failures stay `-32602`: unknown tool, malformed
   envelope, and every failure inside rmcp's parameter
   deserialization, which happens before the handler body runs
   (`biomcp` missing `command`, `variant_normalize_car` missing
   `inputs`, `gene_cspec` missing `gene`, `variant_articles` missing
   `items`, `variant_erepo` type mismatches). Convertible sites:
   `search_args`/`checked_text`/`input_error` (shell.rs:649-814), the
   binary-section guard in `get_args` (:871), typed_get field
   validation, `variant_normalize_car` (:1142-1147), `variant_erepo`
   (:1175, :1202, :1211), `gene_cspec` (:1254), `variant_articles`
   (:1310, :1316, :1322), and `variant_article_strategy`
   (:313-328). The raw `biomcp` tool already returns isError results
   for its argument problems.
3. Unknown cursor. Both tools/list implementations reject a non-empty
   `cursor` with `-32602`: the rmcp `ServerHandler::list_tools`
   (shell.rs:1401-1410, legacy session) and the modern stateless
   dispatch (`modern.rs:111`), which the conformance suite actually
   exercises. `resources/list` is in scope the same way (both arms)
   since the spec's pagination text covers it.
4. Tests. The schema-shape test lands beside the existing unit tests
   in `src/mcp/shell.rs`. The modern-path garbage-cursor `-32602` and
   isError assertions land in `tests/test_mcp_2026_protocol.py` (raw
   stdio modern dialect); the legacy-session cursor case uses
   `PaginatedRequestParams::with_cursor` in
   `tests/rmcp_client_contract.rs`. The external
   `@hasmcp/mcp-spec-test` run stays the verification step: after the
   schema change, `variant_erepo` becomes the suite's probe target
   (the only empty-`required` root), `{}` reaches its in-body check,
   and the isError result satisfies the tools/call case that skips
   today. Live provider verification (Anthropic, OpenAI, Gemini)
   remains a recorded residual needing separate authorization.

## Acceptance

- Every tool root carries top-level `properties`; `search` and `get`
  additionally carry `required: ["entity"]`, enum-derived from the
  branch lists.
- In-body argument validations come back as `isError: true` tool
  results; deserialization-level failures stay `-32602`.
- Unknown cursors return `-32602` on both tools/list paths and both
  resources/list paths.
- The new tests cover all three behaviors on the modern path, plus the
  legacy cursor case.

## Review

- Design review: ACCEPT with required changes 2026-09-24 (two P1s —
  the modern tools/list path and the spec-claim re-scoping — plus P2
  factual corrections, all folded in above)
- Code review: pending
