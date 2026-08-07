---
base: 27ccd6123e95f5add988ee26cd3fff15448f7d3e
head: 4efe1ee83ae4e9cb89af3cb8d8f26e5582c6ca5a
---
Drug alias resolution is one-directional: `biomcp get drug pembrolizumab` resolves the brand→generic side, but a user-supplied research code (e.g. `MK-3475`) does not symmetrically resolve to `pembrolizumab`. This breaks the agent ergonomic of "I have a code from a paper, what trial drug is this?" — which is a primary biomcp use case.

Imported from March ticket 310. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/310-make-drug-alias-resolution-bidirectional-research-code-to-inn
