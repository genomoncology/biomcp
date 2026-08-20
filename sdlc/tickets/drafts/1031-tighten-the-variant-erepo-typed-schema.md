---
flow: build
priority: 5
hold: draft for review; do not promote until Ian releases this
---
# Tighten the variant erepo typed schema

The typed MCP tools are described as an interface an agent can rely on, with enums and bounds that stop a model guessing. `variant_erepo` does not hold that line. Its fields are largely nullable strings with no enums, and the mutual exclusivity of `caid`, `caids`, and `gene` is not expressed in the schema at all — an agent can supply all three and only discovers the problem from a runtime error.

The `search` tool shows what the standard should be: per-entity constants, enumerated sections, and bounded page sizes. The value of a typed catalog comes from being uniform. One loose tool teaches a model that the schemas are advisory, and the cost is paid in wasted turns across every tool.

## Done when

- `variant_erepo`'s schema expresses which of `caid`, `caids`, and `gene` may appear together, so an invalid combination is rejected before a call is made rather than after.
- Fields with a known closed set of values carry that set in the schema.
- Bounded fields carry their bounds.
- A survey of the other typed tools is included in the design, naming any that fall below the `search` standard, so it is clear whether this is one outlier or a pattern. Fixing the others is not in scope unless the design shows it is the same change.
