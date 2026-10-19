---
priority: 3
---

# 2263 — Surface MCP argument parse errors to the model

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 5aea853, accept

## Outcome

Changes: `sdlc/tickets/2258-admit-the-0-9-4-release-candidate.md`, `sdlc/tickets/2262-set-stable-0-9-4-metadata-and-dispositions.md`, `Cargo.toml`, `Cargo.lock`, `src/mcp/shell.rs`, `src/mcp/shell/arguments.rs`, `src/mcp/shell/tests/ticket_1120.rs`, `src/mcp/shell/typed_get_tests.rs`, `tests/rmcp_client_contract.rs`, `sdlc/planning/adr/0002-flat-mcp-tool-schemas-for-function-calling.md`, `CHANGELOG.md`, `sdlc/tickets/2263-surface-mcp-argument-parse-errors-to-the-model.md`.

A malformed MCP tool call, such as `gene_cspec` without `gene`, returns an `isError` tool result whose message names the bad argument, so the calling model can correct itself. The `variant_erepo` root schema can no longer drift from its branches.

## Evidence

Residuals of 1251 in `sdlc/issues/2026-09-24-mcp-schema-follow-ups-after-1240.md`: parse failures inside rmcp's `Parameters` stay protocol errors (-32602) that the model never sees; and the `variant_erepo` flat root restates its property list by hand.

## Keeps

Valid calls are unchanged. Unknown-field rejection keeps working. Keep the recorded -32602 channel only where the spec requires a protocol error.

## Proof

A stdio test sends `gene_cspec` without `gene` and receives an `isError` result naming `gene`. A test derives the `variant_erepo` root property set from its branch types and asserts the published schema matches.

## Implementation evidence

Fresh developer stopped clean at 78461b10230be2fe68e72acfcd770df49170be96. Current-main baseline 15a04fdd7 reproduced gene_cspec with an empty argument object returning protocol -32602 and missing gene. Job 2263/1792436017503-5865 made the outside-in regression fail for that reason. Shared typed parsing now retains argument paths, and the known-tool boundary returns model-visible isError results; unknown-tool and required protocol errors remain on their channel. ERepo root properties derive from the real typed argument source rather than a second handwritten map.

Job 2263/1792436323841-62490 passed five targeted behavior and schema contracts, complete published-tools-catalog equality against baseline, a modern stdio probe and make lint. Job 2263/1792436533286-8289 passed formatting and tracked-text checks including the new module. Failed intermediate compile and catalog probes remain retained. No full gate or live-provider checks were claimed.

Keep: experiments/2078-biomcp-2263-mcp-arguments — retain baseline, red and green results, tested patches, catalog comparison, commands and source hashes.

This changes dependency declarations and must land alone through the existing gate-definition rule. Stable metadata 2262 waits for this source fix.
