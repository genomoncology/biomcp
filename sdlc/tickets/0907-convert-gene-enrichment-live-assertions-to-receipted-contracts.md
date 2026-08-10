---
flow: build
priority: 8
deps: ["0651", "0906"]
---
# Convert gene enrichment live assertions to receipted contracts

This is the optional-enrichment slice of superseded ticket 0676. It follows
0906's file split and owns QuickGO, HPA, ChEMBL, NIH Reporter, and GTR blocks
in spec/entity/gene-live.md.

## Done when

Each optional section's success, absence, not-requested, not-configured, and
provider-failure behavior runs routinely against local provider-faithful
responses. Proof covers the section CLI controls, exact consumed plans,
observed local requests, production decoders and orchestration, typed source
outcomes, and JSON and Markdown rendering.

The converted blocks move into gene.md. gene-live.md leaves the live registry
and is removed only when no live assertion remains. The main runner, the
separate NIH disease/gene lane, Makefile, and architecture inventory are
reconciled rather than silently duplicating the NIH proof.

Existing provider request and decoder tests remain. Add only missing layers.
Captures must come through production requests. No production enrichment
policy changes to make a fixture pass.

## Authorized test changes

The named enrichment blocks, fixture routing, receipts, and registry entries
may be restated in design commits. Mechanical construction fixes may land
with implementation while keeping assertions intact.

The src line ceiling may rise by at most 150 lines.
