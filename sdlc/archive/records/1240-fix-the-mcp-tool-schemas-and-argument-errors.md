---
base: cd18cefa
head: 544db639
---

Fixed the MCP tool schemas and argument errors, from the conformance
follow-ups issue.

Every tool root now carries top-level `properties`, so model providers
that build arguments from top-level properties instead of reading
`oneOf` branches see the real shape. `search` and `get` additionally
carry `required: ["entity"]` with an entity enum derived mechanically
from the same lists that build their branches (a shared `ENTITIES`
const for search; the `typed_get_capabilities()` mapping for get), so
the root enum cannot drift from the branch consts. `variant_erepo`
publishes the union of its selector fields as optional properties while
keeping its branches for validation.

In-body argument validation now returns a tool result with `isError`
and the message as text — the spec's SHOULD for tool-originated errors,
chosen deliberately so a model can read the bounds and self-correct —
while protocol-shape failures stay `-32602`: unknown tool, malformed
envelope, and everything rmcp rejects during parameter deserialization
before the handler body runs (a missing `command`, `inputs`, `gene`,
`items`, or a type mismatch), which cannot convert without
restructuring rmcp. A wrapper maps invalid-params errors from the
search, get, and variant_articles bodies; the eight direct validation
returns in variant_normalize_car, variant_erepo, gene_cspec, and
variant_articles convert to error results. Both `tools/list` paths
(the rmcp legacy session and the modern stateless dispatch) and both
`resources/list` paths reject a non-empty cursor with `-32602`; the
server never paginates, so any cursor is unknown.

After the schema change the conformance probe's selection flips to
`variant_erepo` (the only empty-`required` root), whose `{}` reaches
the in-body check and now returns the isError result the suite
accepts — the tools/call case that skipped today becomes verifiable.
Tests: schema-shape pins in shell.rs (root enum equals the branch
consts in order; the erepo union; no erepo required), the modern-
dialect garbage-cursor and isError tests in test_mcp_2026_protocol.py,
the legacy cursor test through the builder in rmcp_client_contract.rs,
and the two contract-client assertion updates (binary-download and
search-bounds now assert error results).

Evidence: design ACCEPT after two reviews (the modern list path and
the spec-claim re-scoping); code review REJECT once with three P0s
(a test that did not compile, a stale unit pin, and two unchanged
contract assertions) all fixed and verified; four gate cycles caught
formatter shapes, a non-exhaustive struct expression, and the stale
adverse-event pin; final yellow gate at 544db639 — lint, test, and
spec OK, with the tools-list measure at 17,175 bytes / 4,365 tokens,
within the released ceilings even with the added root properties.

Residuals: live verification against Anthropic, OpenAI, and Gemini
stays recorded and needs separate authorization; the external
@hasmcp/mcp-spec-test rerun belongs to the verification leg; a
non-string cursor is still silently ignored on the modern path
(the legacy arm rejects it via deserialization).
