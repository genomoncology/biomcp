---
flow: build
priority: 9
deps: ["0928", "0932"]
---
# Make typed MCP schemas entity-specific

The typed `get` schema currently exposes one union of every entity's section
names and validation accepts a section known to any entity. The typed `search`
tool exposes only generic query and limit fields, hiding useful filters behind
the raw shell-shaped escape hatch.

## Schema contract

Generate discriminated `oneOf` branches from the authoritative command/entity
catalog. Each `get` branch contains one literal entity and only that entity's
identifier, section names, and bounded options. Each `search` branch contains
one literal entity and the filters the shipped typed path actually supports.
An entity/section or entity/filter mismatch fails JSON Schema validation before
dispatch; runtime validation enforces the same rule.

Keep the common typed surface intentionally smaller than the complete CLI.
Include high-value bounded filters already represented by stable CLI types;
do not duplicate mutating commands, binary downloads, local paths, or every
raw option. The `biomcp` tool remains the documented escape hatch for valid
commands not in the typed schema.

## Done when

- Generated schemas cover every intentionally typed search/get entity and no
  section appears under the wrong entity.
- Positive examples for article, trial, variant, gene, protein, PGx, GWAS, and
  author reach the expected command exactly once through local fixtures.
- Cross-entity sections, unknown filters, raw byte downloads, and oversized
  values fail before command dispatch.
- Schema size remains inside ticket 0932's aggregate tools/list budget.
- MCP reference examples are generated or checked from the same branches, and
  every example validates against the published schema.

## Authorized test changes

Design commits may restate typed search/get schemas and validation in
`src/mcp/shell.rs`, MCP contract-client fixtures, MCP specs, and generated MCP
reference examples. Existing specialized ClinGen tools, raw allowlist, result
rendering, and safety annotations remain covered.

The src line ceiling may rise by at most 260 lines.
