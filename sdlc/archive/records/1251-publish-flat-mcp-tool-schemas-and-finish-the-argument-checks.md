---
base: fb79e413
head: 74f600db
---

Published flat MCP tool schemas and finished the argument checks, from
the schema follow-ups issue.

The three tools that still wrapped arguments in a top-level `oneOf` —
search, get, and variant_erepo — now publish one flat object root: the
entity enum plus the union of every branch's properties, merged under
one collision rule (same-named enums union into one, so `source` lists
all eight author/article/trial values; string-or-array fields like
`disease` and `drug` publish `["string","array"]` with the item
schema). The flat lists are built from the same consts the branches
validate against, so root and body cannot drift, and a hoisted
`ENTITIES` const feeds the root enum, `search_args`, and the rejection
message. OpenAI and Gemini function calling, which reject top-level
`oneOf`, now see every field. A present non-integer `limit` or
`offset` rejects with an isError result naming the field (explicit
null stays lenient, recorded). `variant_erepo` gained
`deny_unknown_fields`: unknown fields now reject with rmcp's `-32602`
— the recorded pre-body channel — rather than being silently ignored.
The drift tripwire tests were rewritten to assert the flat unions, a
catalog walk checks all seven tools (root object, non-empty
properties, no root combinator, required ⊆ properties), the modern
and rmcp template/prompt list paths reject unknown cursors, and the
changelog gained the two (1240) Changed bullets plus (1251). The
decision is recorded in
`sdlc/planning/adr/0002-flat-mcp-tool-schemas-for-function-calling.md`:
flat unions for function-calling compatibility, validation stays
in-body as spec-permitted isError results, rmcp parameter
deserialization failures stay `-32602`.

Every consumer of the old root `oneOf` was updated: the streamable-
HTTP contract example, the gene and drug spec pages, the release
smoke script, the trial help contract (including its stale sources
equality), and the contract-client crate's branch-count assertions.

Evidence: design REJECT once with three P1s (the `-32602`/isError
contradiction, the missing collision rule that would have hidden
valid search values on exactly the providers this serves, and `region`
misattributed to get) — folded, re-reviewed ACCEPT; code review REJECT
twice (first the stale oneOf consumers across five surfaces, then a
line-order defect in the mcp spec page) plus the contract-client
assertions found by the gate — all fixed and verified; yellow gate at
74f600db — lint, test, and spec OK after three cycles (the
contract-client assertion, the ADR's package count, both folded).

Residuals: the rmcp `Parameters` wrapper (turning pre-body
deserialization failures into isError results) and the conformance
`--tool-args` rerun stay recorded; the erepo root schema remains a
hand-restated field list (pinned by test, not derived from a shared
const); the erepo CAID mode ignores `limit` (pre-existing, noted).
