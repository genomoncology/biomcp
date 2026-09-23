# MCP tool schemas and argument errors

Filed 2026-09-23 from an independent review of `v0.9.0..f2549676`. Follows ticket 1223.

## Top-level oneOf may be rejected by model providers

`search`, `get`, and `variant_erepo` publish a root of only `type` and `oneOf`, with no top-level `properties` (`src/mcp/shell.rs:291`, `:301`, `src/mcp/shell/typed_get.rs:81`). Some LLM APIs are reported to refuse tools whose schema has `oneOf`, `anyOf`, or `allOf` at the top level, failing the whole request. This shape has shipped since 2026-08-12 and no record weighs provider compatibility. Not yet tested live.

The conformance suite's tools/call case stays unverified for the same reason: it builds arguments from top-level `properties`, finds none, sends `{}`, and gets "entity must be a string".

Fix: make one real tool call through Anthropic, OpenAI, and Gemini. If any rejects the schema, add top-level `properties.entity` with the entity enum and `required: ["entity"]`, or flatten each tool into one object. Record the result.

## Argument errors come back as protocol errors

`tools/call search {}` returns JSON-RPC `-32602`. The MCP spec asks for input validation failures as a tool result with `isError: true`, so the model sees the message and retries. Check the current spec text and return argument errors that way.

## Unknown cursor accepted

A garbage `tools/list` cursor returns the full list. Reject it with `-32602`. This is the one conformance recommendation not met.

## Unverified cases

Prompts and the legacy-version client are correctly out of scope. Cancelled-subscription teardown has not been investigated.
