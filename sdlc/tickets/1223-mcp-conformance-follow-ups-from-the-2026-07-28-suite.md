---
flow: build
priority: 3
deps: []
---

# 1223: MCP conformance follow-ups from the 2026-07-28 suite

## Goal

Close the actionable cases the 0.1.5 conformance suite flags against 0.9.0 after the eight findings in GitHub issue #248 were fixed. The tool-schema cases close by declaring `type: object`; the subscription case closes by a fixed shape or a recorded decision; the not-verified and recommended cases are recorded. The reporter's cases pass; these are new.

## Current Facts

Run of `npx @hasmcp/mcp-spec-test@latest -c "docker run -i --rm ghcr.io/genomoncology/biomcp:0.9.0 serve"` (suite 0.1.5): 28 passed, 3 failed, 5 not verified, 1 recommended not met.

- Failed, `tools/list returns schema-conformant tools`: `search: inputSchema.type must be "object"`. The search schema rewrite at `src/mcp/shell.rs:284-291` (and the get rewrite at `src/mcp/shell/typed_get.rs:81`) replaces the generated schema with a bare `{"oneOf": branches}`; only those two rewrites drop the top-level type, and `src/mcp/catalog.rs:12-27` puts them at `tools[1]` and `tools[2]`.
- Failed, `subscriptions/listen acknowledges only the opted-in notification types`: the opted-in type must be acknowledged, got `false`. The current shape is deliberate: `src/mcp/shell/modern.rs:152-160` returns `notifications/subscriptions/acknowledged` with an empty `notifications` object, pinned by `tests/test_mcp_2026_protocol.py:191-216`.
- Not verified: `tools/call` returned a schema-conformant `CallToolResult` (the probe's `search` call declined with `entity must be a string`); `prompts/list` and `prompts/get` (no prompts capability is advertised); an older-version client receives no newer-revision fields (a stateless `tools/list` on 2025-11-25 is rejected with `UnsupportedProtocolVersionError`); a cancelled subscription teardown result.
- Recommended, not met: an unrecognised pagination cursor is answered rather than rejected with `-32602`.

## Design

- Make the `search` and `get` tool schemas declare `"type": "object"` at the top level alongside `oneOf` (`src/mcp/shell.rs:284-291`, `src/mcp/shell/typed_get.rs:81`), matching `variant_erepo` at `src/mcp/shell.rs:300`. Update `scripts/release-smoke.sh:274-275`, which reads `inputSchema.properties.entity` and stays broken with a bare `oneOf`, to read the matching branch.
- Settle the `subscriptions/listen` acknowledgment contract from the 2026-07-28 spec text: quote the sentence that decides whether an implemented type must be acknowledged, then either change `src/mcp/shell/modern.rs:152-160` and `tests/test_mcp_2026_protocol.py:191-216` to match, or record the divergence and its rationale in this ticket and the record. The external suite cannot be changed here.
- Decide the stateless 2025-11-25 `tools/list` case: serve it or record the session requirement as deliberate, since `src/mcp/shell/modern.rs:53-59` currently rejects any version other than 2026-07-28 and `src/mcp/shell/pre_session.rs:76-77` routes metadata-bearing requests to that path.
- Pin whatever is decided in `spec/surface/mcp.md` so the suite's cases cannot regress silently.

## Acceptance

1. A manual suite run against a working-tree build, off the gate host, reports no failures in `tools/list` schema conformance or the official-SDK tool listing, with the raw summary line pasted into the record. Use `npx -y @hasmcp/mcp-spec-test@latest -c "$PWD/target/spec/biomcp serve"` after `make prepare-spec`; the Dockerfile needs staged per-arch binaries, so a local image is optional.
2. The `subscriptions/listen` case passes, or the divergence is recorded with the quoted spec sentence and the pinning test updated or explicitly reaffirmed.
3. The stateless 2025-11-25 decision is recorded and pinned, or moved to Out of scope with the reason.
4. `spec/surface/mcp.md` pins the tool schema shape and the subscription behavior, and `scripts/release-smoke.sh` reads the schema it now advertises.
5. `make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- The eight findings in issue #248, which 0.9.0 already fixes.
- The unrecognised-cursor recommendation (`-32602`); it is a SHOULD, not a conformance failure.
- Adopting the suite in CI; the build lane stays offline and the suite run is manual.

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

## Decisions

### `subscriptions/listen` acknowledgment

The 2026-07-28 published schema defines the acknowledgment payload as:

> The subset of requested notification types the server agreed to honor. Only includes notification types the server actually supports; if the client requested an unsupported type (e.g., `promptsListChanged` when the server has no prompts), it is omitted from this set.

(`SubscriptionsAcknowledgedNotificationParams.notifications` description in
`spec/2026-07-28/schema.json`, vendored by `@hasmcp/mcp-spec-test@0.1.5`.)

An opted-in type the server supports therefore belongs in the acknowledgment,
and the empty object was a divergence. `src/mcp/shell/modern.rs` now reports
`toolsListChanged` and `resourcesListChanged` when the client opts in, because
the server advertises tools and resources. It omits `promptsListChanged` (no
prompts capability) and `resourceSubscriptions` (its resources never update),
and it never acknowledges a type the client did not request. Pinned by
`tests/test_mcp_2026_protocol.py` and `spec/surface/mcp.md`.

### Stateless 2025-11-25

Per-request metadata is the 2026-07-28 stateless mode, so a request whose
metadata names a legacy revision is not a legacy session. BioMCP keeps rejecting
it with `-32022` and serves legacy revisions only through `initialize`. Recorded
in `spec/surface/mcp.md`, which pins the rejection, and by
`tests/test_mcp_2026_protocol.py`.

## Review

- Design review: pending
- Code review: pending
