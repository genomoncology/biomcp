---
flow: build
priority: 3
deps: []
---

# 1223: MCP conformance follow-ups from the 2026-07-28 suite

## Goal

Close the cases the 0.1.5 conformance suite still flags against 0.9.0 after the eight findings in GitHub issue #248 were fixed. The reporter's cases pass; these are new.

## Current Facts

Run of `npx @hasmcp/mcp-spec-test@latest -c "docker run -i --rm ghcr.io/genomoncology/biomcp:0.9.0 serve"` (suite 0.1.5): 28 passed, 3 failed, 5 not verified, 1 recommended not met.

- Failed, `tools/list returns schema-conformant tools`: `search: inputSchema.type must be "object"`.
- Failed, `a stock official-SDK client can list tools`: `tools[1].inputSchema.type` and `tools[2].inputSchema.type` are not `"object"`.
- Failed, `subscriptions/listen acknowledges only the opted-in notification types`: the opted-in type must be acknowledged, got `false`.
- The schema rewrite at `src/mcp/shell/typed_get.rs:81` replaces the generated schema with a bare `{"oneOf": branches}`, and `subscriptions/listen` answers with `notifications/subscriptions/acknowledged` and an empty `notifications` object (`src/mcp/shell/modern.rs:152-157`).
- Not verified: `tools/call` returned a schema-conformant `CallToolResult` (the probe's `search` call declined with `entity must be a string`); `prompts/list` and `prompts/get` (no prompts capability is advertised); an older-version client receives no newer-revision fields (a stateless `tools/list` on 2025-11-25 is rejected with `UnsupportedProtocolVersionError`); a cancelled subscription teardown result.
- Recommended, not met: an unrecognised pagination cursor is answered rather than rejected with `-32602`.

## Design

- Make every advertised tool's `inputSchema` an object with `"type": "object"`, and pin the whole tool list against the 2026-07-28 schema so SDK clients can parse it.
- Decide and implement the `subscriptions/listen` acknowledgment contract, or record why the current shape is correct and adjust the suite expectation.
- Decide the stateless 2025-11-25 `tools/list` case: serve it or document the session requirement.
- Pin whatever is decided in `spec/surface/mcp.md` so the suite's cases cannot regress silently.

## Acceptance

1. The suite reports no failures in `tools/list` schema conformance or the official-SDK tool listing.
2. The `subscriptions/listen` case is either passing or explicitly decided and recorded.
3. `spec/surface/mcp.md` pins the tool schema and the subscription behavior.
4. `make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- The eight findings in issue #248, which 0.9.0 already fixes.
- Adopting the suite in CI; the build lane stays offline.

## Complexity

- Contract score: 2 (a new compatibility decision for the `subscriptions/listen` acknowledgment contract and the stateless 2025-11-25 case)
- State and timing score: 1 (protocol sessions, subscriptions, and cursors)
- Reach score: 1 (the MCP public surface in one module)
- Proof score: 2 (the conformance suite plus official-SDK interop across several cases)
- Cost of error score: 1 (client-visible breakage for MCP clients)
- Total: 7
- Minimum level floor: none (no durable state or credential changes)
- Final level: 3
- Reasons: protocol-level compatibility decisions with multi-case external proof
- Selected model: gpt-5.6-sol, medium reasoning (level 3 implementer)

## Review

- Design review: pending
- Code review: pending
