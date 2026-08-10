---
flow: build
priority: 8
deps: ["0651", "0914", "0915"]
---
# Convert gene identity live assertions to receipted contracts

This is the identity slice of superseded ticket 0676. It owns MyGene
resolution and the base gene card in spec/entity/gene.md, not optional
enrichment sections.

## Done when

Symbol and alias resolution, base identity fields, source attribution, absent
identity, and the base JSON and Markdown card run routinely against a
receipted MyGene response. Proof covers CLI parsing, the exact consumed
RequestPlan, the request observed by a local fixture, the production decoder
and entity transform, and rendering.

Existing MyGene plan and decoder tests remain. Add only missing execution,
receipt, orchestration, process, and render proof.

## File ownership

The design stage authors the replacement assertions. gene.md becomes routine
with the identity blocks. Optional QuickGO, HPA, ChEMBL, NIH Reporter, and GTR
live blocks move verbatim to gene-live.md for 0907. Registries, Makefile, and
architecture inventory agree.

The capture is real, dated, receipted, and produced through the production
request path. No alias policy, enrichment behavior, or unrelated workflow
guidance changes belong here.

## Authorized test changes

The identity spec blocks, MyGene fixture/receipt, and registry entries may be
restated in design commits. Mechanical construction fixes may land with code
without weakening behavioral assertions.

The src line ceiling may rise by at most 100 lines.
