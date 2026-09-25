# 0002 — Publish flat MCP tool schemas for function calling

- Status: accepted
- Date: 2026-09-25
- Relates to: tickets 1240 and 1251; the 2026-07-28 MCP conformance suite

## Context

Model providers build tool arguments from the top-level `properties`
of a tool's input schema. OpenAI and Gemini function calling reject a
top-level `oneOf` outright, and a client that drops the construct sees
only the shared `entity` field and never learns about `query` or `id`.
Until ticket 1240, `search`, `get`, and `variant_erepo` published
exactly that shape: a thin root with `entity` plus a `oneOf` of
per-entity branches. The 1240 review drove a live server and confirmed
all seven tools needed top-level properties; the schema follow-up
review confirmed the remaining `oneOf` wrappers still break the two
largest providers.

## Decision

Every tool publishes one flat root: `type: object`,
`additionalProperties: false`, and the merged union of every branch's
properties, with no `oneOf`, `anyOf`, or `allOf` at the root. The root
is descriptive; the body stays prescriptive. The per-entity checks in
the handler bodies already reject bad combinations with `isError` tool
results (ticket 1240, spec-permitted for tool-originated errors), so
flattening loses no enforcement.

The merged union follows one collision rule: same-named fields keep
one schema when identical; `enum`/`const` values union into one `enum`
(`source` merges the author, article, and trial source enums; `get`
sections union every entity's section names); a field that is text on
some branches and a list on others (`disease`, `drug`) publishes
`type: ["string", "array"]` with both sides' constraints. The merged
root can only be wider than any single branch, never narrower. The
flat lists are derived from the same constants and capability tables
the branch builders and the body checks use, so they cannot drift.

Failures that rmcp raises while deserializing `Parameters` (a missing
required field, a wrong type, and — since ticket 1251 — an unknown
`variant_erepo` field) stay `-32602` protocol errors, because they run
before the handler body and rmcp's wrapper cannot be converted without
restructuring rmcp. The MCP specification allows both channels; the
model sees the message either way.

## Options weighed

**Keep the `oneOf` branches.** Costs compatibility with the two
largest providers. Nothing else is gained; the branches duplicate
information the body already enforces.

**Publish the union and drop per-entity validation.** Would make the
schema the only contract. Rejected: per-entity bounds (NCI's single
mutation rule, gwas offset limits, per-entity section sets) cannot be
expressed in one flat schema without reintroducing branching.

**Flat root, prescriptive body (chosen).** Costs a schema that is
wider than any one entity accepts, so a model may send `article` plus
`query` and receive an `isError` naming the bad field, then
self-correct. Buys provider compatibility and one self-describing
surface per tool.

## What this does not change

Typed argument validation, its messages, and its `isError` results are
unchanged. The branch builders remain the validation source of truth
and are pinned by tests. Cursor rejection on every list method stays
`-32602`.

## Reversal

Overturning this decision means a superseding ADR restoring branching
roots and re-verifying provider compatibility. The merge helpers and
the flat-root tests would be retired with it.
