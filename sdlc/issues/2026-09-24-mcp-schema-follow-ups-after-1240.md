# MCP schema follow-ups after ticket 1240

Filed 2026-09-24 from the review of ticket 1240 (merge 60ecdb66). The reviewer drove a debug build over stdio and ran `@hasmcp/mcp-spec-test` 0.1.5: 9 passed, 0 failed, 2 not verified. All seven tools have a root `"type":"object"` and top-level `properties`, and `search` and `get` carry the `entity` enum and `required`.

## Should-fix

- `search`, `get`, and `variant_erepo` still publish a top-level `oneOf` (`src/mcp/shell.rs:289-331`, `src/mcp/shell/typed_get.rs:81-88`). OpenAI and Gemini function calling reject top-level `oneOf`. A client that drops it sees only `entity` and never learns about `query` or `id`. Publish a flat root: the `entity` enum plus the union of every branch's properties, with no `oneOf`. `search_args` (`shell.rs:713`) and `get_args` already validate each branch and now return `isError`, so nothing loses enforcement. Record the choice in an ADR.
- A wrong-type `limit` or `offset` is silently ignored (`shell.rs:741-742`). `{"entity":"gene","query":"BRAF","limit":"abc"}` ran with the default. Reject a present non-integer through `input_error`.
- `variant_erepo` accepts unknown fields although its branches declare `additionalProperties:false`. The struct at `shell.rs:62-64` lacks `#[serde(deny_unknown_fields)]`.
- No test covers every tool. `shell.rs:1780-1831` checks three tools, `tests/test_mcp_tool_catalog.py:156` checks only that each schema is a dict, and the release smoke (`scripts/release-smoke.sh:280-282`) checks only search and get. Loop over the catalog and assert root `type=="object"` and a `properties` object for every tool.
- The CHANGELOG has no 1240 entry. Two client-visible changes need a "Changed" line: argument errors moved from `-32602` to `isError` results, and a non-empty cursor that used to return the full list is now rejected.

## Minor

- Parse failures inside rmcp's `Parameters` stay `-32602`, for example `gene_cspec` without `gene`. The spec allows it, but the model never sees the message. A wrapper can turn them into `isError`.
- The "invalid typed search entity" message should list the valid entities.
- `resources/templates/list` (`src/mcp/shell/modern.rs:124`) and `prompts/list` accept a garbage cursor. Call `reject_unknown_cursor` there too.
- The conformance tools/call case sends `{}` to `variant_erepo`, gets an `isError` result, and passes on the envelope alone. Pass `--tool-args` with a known-safe call and record the run.
- Ticket 1240 still says "Code review: pending".

## Resolved

Ticket 1251. Flat roots for search, get, and variant_erepo with a
collision rule; limit/offset type rejection; deny_unknown_fields on
erepo (-32602, the recorded pre-body channel); the catalog walk; the
cursor minors; ADR 0002; the changelog entries; every oneOf consumer
updated. Residuals: the rmcp Parameters wrapper, the conformance
--tool-args rerun, the erepo hand-restated root list, the CAID limit
ignore. See
`sdlc/records/1251-publish-flat-mcp-tool-schemas-and-finish-the-argument-checks.md`.

Resolved by 2238: CLI and typed MCP reject supplied limit or offset in CAID mode before provider access, including explicit default values. Omitted allele calls remain valid; gene paging retains its defaults and continuation behavior. The Parameters wrapper, conformance tool-arguments rerun and hand-restated root list remain separate residuals.

## Release dispositions (2026-10-19)

- Flat roots, wrong-type paging, unknown fields, every-tool catalog coverage and changelog omissions: FIXED by 1251 with the separate evidence in the cited record.
- Cursor rejection and valid-entity messages: FIXED by 1251; retain its recorded argument-channel distinction.
- CAID paging: FIXED by 2238 at 28036ce3a. Explicit limit or offset is rejected before provider access; omitted allele paging and gene defaults remain valid.
- Parameters wrapper: FIXED by 2263 at df2319982. Malformed known-tool arguments return model-visible isError results naming the bad argument; required protocol errors retain their protocol channel.
- Conformance tools/call arguments: retain the separate 1251 residual; an envelope-only conformance pass does not prove the known-safe tool call requested here.
- Hand-restated ERepo root list: FIXED by 2263 at df2319982. Published root properties derive from the typed argument source, and the recorded catalog comparison preserves valid-call schemas.
- Ticket review wording: HISTORICAL. The 1251 disposition supersedes 1240's open-era review text.
