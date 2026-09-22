---
flow: build
priority: 3
deps: []
---

# 1223: MCP conformance follow-ups from the 2026-07-28 suite

## Goal

Close the cases the 0.1.5 conformance suite still flags against 0.9.0 after the eight findings in GitHub issue #248 were fixed. The reporter's cases pass; these are new.

## Current Facts

- `npx @hasmcp/mcp-spec-test@latest -c "docker run -i --rm ghcr.io/genomoncology/biomcp:0.9.0 serve"` reports 28 passed, 3 failed, 5 not verified, 1 recommended not met.
- Failed: `tools/list` returns tools whose `inputSchema.type` is missing; a stock official-SDK client cannot list tools for the same reason (two tools reported, one named `search`).
- Failed: `subscriptions/listen` does not acknowledge only the opted-in notification types.
- Not verified: a stateless `tools/list` on 2025-11-25 is rejected with `UnsupportedProtocolVersionError`; prompts are not advertised; two subscription teardown cases.
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

## Review

- Design review: pending
- Code review: pending
