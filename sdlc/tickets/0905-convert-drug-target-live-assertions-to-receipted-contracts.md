---
flow: build
priority: 8
deps: ["0651", "0904"]
---
# Convert drug target and interaction live assertions to receipted contracts

This is the target and interaction slice of superseded ticket 0675. It follows
0904's file split and owns the remaining ChEMBL and DDInter blocks in
spec/entity/drug-live.md.

## Done when

Target selection, interaction attribution, local-bundle provenance, empty
results, and unavailable outcomes run routinely against local
provider-faithful ChEMBL bytes and the bounded DDInter fixture/bundle. The
proof covers CLI parsing, exact consumed plans, observed local requests where
HTTP is used, production decoding/orchestration, and JSON and Markdown.

The converted assertions move into spec/entity/drug.md. drug-live.md leaves
the live registry and is removed only when no live block remains. The runner,
Makefile, and architecture inventory must name the same paths.

Existing construction, decoder, and local-bundle tests remain. Add only
missing layers. Captures come through production requests; no production
behavior changes to fit fixtures.

## Authorized test changes

The ChEMBL/DDInter drug spec blocks, fixture routing, receipts, and registry
entries may be restated in design commits. Mechanical construction fixes may
land with implementation while preserving assertions.

The src line ceiling may rise by at most 120 lines.
