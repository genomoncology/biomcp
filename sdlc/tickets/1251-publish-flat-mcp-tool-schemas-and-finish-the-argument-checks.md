# Publish flat MCP tool schemas and finish the argument checks

From sdlc/issues/2026-09-24-mcp-schema-follow-ups-after-1240.md. The
reviewer drove the live server: all seven tools have root
`type: object` and top-level properties, but three tools still wrap
their arguments in a top-level `oneOf`, which OpenAI and Gemini
function calling reject.

## Problem

1. `search`, `get`, and `variant_erepo` publish a top-level `oneOf`
   (`src/mcp/shell.rs:289-331`, `src/mcp/shell/typed_get.rs:81-88`).
   A client that drops the construct sees only `entity` and never
   learns about `query` or `id`.
2. A wrong-type `limit` or `offset` is silently ignored
   (`shell.rs:741-742`): `{"entity":"gene","query":"BRAF","limit":"abc"}`
   runs with the default.
3. `variant_erepo` accepts unknown fields: the struct at `shell.rs:62-64`
   lacks `#[serde(deny_unknown_fields)]`.
4. No test walks every tool in the catalog; three are pinned in
   `shell.rs:1780-1831`, the catalog test checks only dict-ness, and
   the release smoke checks two.
5. The CHANGELOG has no 1240 entry: argument errors moved from
   `-32602` to `isError` results, and a non-empty cursor that used to
   return the full list is now rejected. Both are client-visible.

## Design

1. Flat roots. `search` publishes `entity` (enum, required), `query`
   (string, required), `limit`, `offset`, and every other field any
   branch declares — one flat `properties` object, no `oneOf`. `get`
   publishes `entity` (enum, required), `id` (required), `sections`,
   `region`, and the rest of the branch union. `variant_erepo`
   publishes the union of its selector fields (already done in 1240)
   and drops its `oneOf`. Branch-level validation is unchanged: the
   in-body checks already reject bad combinations with isError
   results, so nothing loses enforcement — the schema is descriptive
   at the root, prescriptive at the body. Build the flat lists from
   the same consts the branches use so they cannot drift (the 1240
   pattern). Record the decision in an ADR
   (`sdlc/planning/adr/0002-flat-mcp-tool-schemas-for-function-calling.md`):
   root schemas are flat unions for OpenAI/Gemini compatibility;
   validation stays in the body as spec-permitted isError results;
   rmcp parameter deserialization failures stay `-32602`.
2. Reject a present non-integer `limit`/`offset` through the existing
   `input_error` path (isError result naming the field and the
   expected type). Absent values keep their defaults.
3. Add `#[serde(deny_unknown_fields)]` to the `variant_erepo`
   argument struct and an isError test for an unknown field.
4. One catalog-walking test: loop over every tool the server lists,
   assert root `type == "object"`, a non-empty `properties` object,
   no `oneOf`/`anyOf`/`allOf` at the root, and that every `required`
   name exists in `properties`. Put it where the catalog is already
   tested (`tests/test_mcp_tool_catalog.py`) driving the real schema
   builder, not a fixture copy.
5. CHANGELOG Unreleased: two Changed bullets tagged (1240) — argument
   validation errors now return isError tool results instead of
   protocol errors, and an unknown cursor on tools/list or
   resources/list is rejected — plus one (1251) bullet for the flat
   schemas.

Minor items from the issue folded in where cheap: the "invalid typed
search entity" message lists the valid entities; `resources/templates/list`
and `prompts/list` reject a garbage cursor through the existing
`reject_unknown_cursor`. The rmcp `Parameters` wrapper and the
conformance `--tool-args` rerun stay recorded residuals, not code
here.

## Acceptance

- The catalog walk passes for all seven tools and fails if any root
  regains a `oneOf`.
- `limit:"abc"` returns an isError result naming limit; an unknown
  `variant_erepo` field returns an isError result.
- The ADR lands with the decision and its trade-offs.
- Yellow gate green at the head SHA.

## Review

- Design review: pending
- Code review: pending
