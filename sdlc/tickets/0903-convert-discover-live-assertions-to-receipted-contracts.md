---
flow: build
priority: 7
deps: ["0651", "0914", "0915"]
---
# Convert discover live assertions to receipted contracts

This is the discover slice of superseded ticket 0666. It owns only
spec/surface/discover.md, OLS4, and DiscoverRequest orchestration.

## Done when

The diabetes identity route, relational redirect, and genuine no-match
article guidance in spec/surface/discover.md run routinely against local
provider-faithful responses. Use SCENAR therapy for the no-match case; OLS4
was observed returning zero rows for it. Do not reuse the old
not-a-biomedical-concept seed, which returns real NCIT matches.

Proof covers CLI parsing, the exact OLS4 RequestPlan, the request observed by a
local fixture, the production decoder and DiscoverRequest orchestration, and
the JSON and Markdown output claimed by the spec.

## Capture and lane rules

Record every real response through the production request path. A hand-built
URL is not an eligible receipt. Existing request and decoder tests are not to
be recreated; add only the missing consumed-plan, orchestration, process, and
rendering proof.

The design stage authors replacement assertions. discover.md becomes routine;
any genuinely live remainder moves verbatim to discover-live.md. The runner,
Makefile, and architecture inventories must agree.

No ranking, alias-resolution, result-limit, or other product behavior changes
belong here.

## Authorized test changes

The discover spec blocks, fixture routes, capture receipts, and registry
entries may be restated in design commits. Mechanical construction fixes may
land with implementation while preserving behavior.

The src line ceiling may rise by at most 120 lines.
