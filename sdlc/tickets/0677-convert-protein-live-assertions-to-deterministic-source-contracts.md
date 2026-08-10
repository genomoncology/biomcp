---
flow: build
priority: 9
deps: ["0651"]
---
# Convert protein live assertions to deterministic source contracts

## Done when

The UniProt and ComplexPortal source-backed blocks in
spec/entity/protein.md run routinely against local provider-faithful bytes.
Identity, source attribution, complex membership, empty results, unavailable
outcomes, pagination fields, and the claimed JSON/Markdown cards are covered
without public network access.

## Proof required

UniProt and ComplexPortal already have request and decoder tests. Preserve
them and add only:

- dated real receipts produced by the production request paths;
- consumed entity/orchestration proof;
- local executor proof for any transport behavior not covered centrally;
- process-level CLI parsing and rendering.

Do not change protein filter semantics, pagination semantics, decompression
limits, or add providers in this conversion.

The design stage authors replacement assertions. protein.md becomes routine;
any live remainder moves verbatim to protein-live.md. Registry arrays,
Makefile, and architecture inventory agree.

## Authorized test changes

Design commits may restate the protein spec blocks, UniProt/ComplexPortal
source/entity tests needed for missing proof, fixture routes and receipts, and
registry entries. Mechanical construction fixes may land with implementation
while behavioral assertions remain unchanged.

The src line ceiling may rise by at most 120 lines.
