---
flow: build
priority: 8
deps: ["0651", "0914", "0915"]
---
# Convert VAERS live assertions to deterministic aggregate contracts

## Done when

The VAERS/OpenFDA blocks in spec/entity/vaers.md run routinely against local
provider-faithful data. Vaccine eligibility, aggregate request construction,
aggregate decoding, combined source status, empty results, unsupported input,
and rendered output no longer depend on the public aggregate endpoint.

## Proof required

VAERS already has a pure aggregate RequestPlan and decoder, with broad focused
tests. Preserve them. Add:

- one dated real VAERS aggregate receipt obtained through the production
  request path;
- eligible, empty, and unsupported orchestration cases;
- local proof that the executor consumes the plan if no shared transport
  contract already covers it;
- process-level CLI and JSON/Markdown proof.

The existing CVX local-only seam remains local. Do not add a provider, change
the aggregate model, or alter eligibility to fit a fixture.

The design stage authors replacement assertions. vaers.md becomes routine;
any live remainder moves verbatim to vaers-live.md. Runner, Makefile, and
architecture inventory agree.

## Authorized test changes

Design commits may restate the VAERS spec blocks, VAERS/OpenFDA source and
entity tests needed for missing layers, fixture receipts/routes, and registry
entries. Mechanical construction fixes may land with implementation while
preserving behavior.

The src line ceiling may rise by at most 140 lines.
