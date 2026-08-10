---
flow: build
priority: 8
deps: ["0651", "0914", "0915"]
---
# Convert disease live assertions to receipted contracts

This is the disease slice of superseded ticket 0666. It owns only
spec/entity/disease.md and the disease requests behind it.

## Done when

The MyDisease, Monarch disease, NIH Reporter, and SEER assertions in
spec/entity/disease.md run routinely against local provider-faithful bytes.
The routine proof covers CLI parsing, the exact production RequestPlan,
request execution through a local fixture, the production decoder and
orchestration, and the JSON or Markdown claim made by each converted block.
The public-provider smoke, if retained, runs only in the live verify lane.

## Existing proof to keep

These providers already have request-construction and decoder tests. Inventory
them before adding tests and add only the missing consumed-plan, receipt,
orchestration, CLI-process, and rendering proof. Do not create a second
request-plan model for tests.

Every real capture must be obtained by driving the production request path
through a recording proxy or equivalent source seam. The receipt's method,
path, query, headers, and body must match the RequestPlan. The old 0666
MyDisease capture is not eligible: it used a hand-built bare query and the
wrong page size.

## Spec split

The design stage owns the literal replacement assertions and the established
file split:

- spec/entity/disease.md becomes routine and holds the converted assertions.
- Any genuinely live remainder moves verbatim to
  spec/entity/disease-live.md and stays in the live registry.
- scripts/run-specs.sh, Makefile, and the architecture inventory agree.

No assertion is deleted or weakened merely because an upstream is unreliable.
No production limit or query behavior changes to make a fixture fit.

## Authorized test changes

The design and design-review commits may restate the disease spec blocks,
their fixture routing, capture receipts, and the live/routine registry entries.
Construction-only updates needed to compile an implementation field change may
land with the code as mechanical fixes, with behavioral assertions unchanged.

The src line ceiling may rise by at most 160 lines.
