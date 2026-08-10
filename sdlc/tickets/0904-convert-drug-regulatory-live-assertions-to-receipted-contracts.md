---
flow: build
priority: 8
deps: ["0651", "0914", "0915"]
---
# Convert drug regulatory live assertions to receipted contracts

This is the regulatory and regional slice of superseded ticket 0675. It owns
MyChem, OpenFDA, EMA, and WHO prequalification behavior in
spec/entity/drug.md.

## Done when

Regional selection, provider attribution, successful overlays, empty results,
not-configured outcomes, and provider failures run routinely against local
provider-faithful bytes. The proof covers CLI filters, exact consumed
RequestPlans, observed local requests, production decoding/orchestration, and
the JSON and Markdown claims in the converted blocks.

Inventory existing source planning and decoder tests first. Add only missing
transport, receipt, orchestration, CLI-process, and renderer proof.

## File ownership

The design stage authors the replacement assertions. drug.md becomes routine
with the converted regulatory blocks. The still-live ChEMBL and DDInter blocks
move verbatim to drug-live.md for ticket 0905. Registry arrays, Makefile, and
architecture inventory must agree.

Captures are real, dated, receipted, and produced by the production request
path. No runtime query, limit, or source policy changes to fit a capture.

## Authorized test changes

The named drug spec blocks, fixture routing, receipts, and registry entries may
be restated in design commits. Mechanical construction fixes may land with
implementation without changing assertions.

The src line ceiling may rise by at most 150 lines.
