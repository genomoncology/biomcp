---
flow: build
priority: 5
---
# Make drug alias resolution bidirectional research-code to INN

Drug alias resolution is one-directional: `biomcp get drug pembrolizumab` resolves the brand→generic side, but a user-supplied research code (e.g. `MK-3475`) does not symmetrically resolve to `pembrolizumab`. This breaks the agent ergonomic of "I have a code from a paper, what trial drug is this?" — which is a primary biomcp use case.

Completed under March on 2026-04-26, as March ticket 310. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/310-make-drug-alias-resolution-bidirectional-research-code-to-inn
