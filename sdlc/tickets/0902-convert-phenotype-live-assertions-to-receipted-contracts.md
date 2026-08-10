---
flow: build
priority: 8
deps: ["0651", "0914", "0915"]
---
# Convert phenotype live assertions to receipted contracts

This is the phenotype slice of superseded ticket 0666. It owns only
spec/entity/phenotype.md and the Monarch/HPO requests behind its phrase and
identifier routes.

## Done when

The phrase, direct HPO identifier, no-result, and rendered follow-up behavior
claimed by spec/entity/phenotype.md run routinely against local
provider-faithful responses. The proof covers:

- Clap parsing into the exact production request inputs;
- the consumed Monarch/HPO RequestPlans and observed local HTTP requests;
- production decoding and entity orchestration over recorded bytes;
- stable JSON and Markdown output, including absent and unavailable outcomes.

Existing source request and decoder tests remain the foundation. Add only
missing layers; do not duplicate them under new test-only projections.

## Capture and lane rules

Captures are real, dated, receipted, and produced through the production
request path. Synthetic fixtures remain allowed only for an edge condition a
provider cannot naturally produce, and they must be labeled synthetic.

The design stage authors the replacement assertions and splits the file in the
established shape: phenotype.md becomes routine; any unconverted live blocks
move verbatim to phenotype-live.md. Registry arrays, Makefile, and architecture
inventory must agree.

No runtime behavior changes to satisfy a fixture.

## Authorized test changes

The phenotype spec blocks, their fixture routing, capture receipts, and
live/routine registry entries may be restated in design commits. Mechanical
construction updates required by implementation may land with code without
changing behavioral assertions.

The src line ceiling may rise by at most 120 lines.
