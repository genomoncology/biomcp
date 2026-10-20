---
base: fe3d3cf6d9890bc8895b985a4f0a3adf11e6e0f5
head: 5d69c9dfcd2246d6ec94073257470f8b8b90b64d
---

# Tighten the variant ERepo typed schema

`variant_erepo` now advertises closed CAid, CAid-batch, and gene selector
schemas with nonempty values and bounded batch and paging inputs. Agents can
reject unsupported selector mixtures before dispatching an MCP call.
