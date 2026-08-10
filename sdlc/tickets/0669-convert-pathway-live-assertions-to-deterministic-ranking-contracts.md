---
flow: build
priority: 6
deps: ["0651"]
---
# Convert pathway live assertions to deterministic ranking contracts

## Done when

The source-backed KEGG, Reactome, and WikiPathways assertions in
spec/entity/pathway.md run routinely against local provider-faithful bytes.
They prove alias/ranking behavior, source cards, capability fields, empty
results, unavailable outcomes, and pagination without asserting that a public
provider is reachable today.

## Proof required

The three source modules already have broad RequestPlan and decoder tests.
Inventory and retain them. Add only the missing layers:

- dated real search/detail captures with receipts generated through the
  production request path;
- proof that entity orchestration consumes the production plans;
- one local transport observation where the shared executor is not already
  proven;
- process-level CLI parsing and stable JSON/Markdown rendering.

A synthetic fixture may cover an edge a real provider cannot naturally
produce, but it must be labeled synthetic and cannot replace the real shape
anchor. No ranking or provider policy changes belong in this conversion.

The design stage authors the replacement spec assertions. pathway.md becomes
routine; any genuinely live remainder moves verbatim to pathway-live.md.
scripts/run-specs.sh, Makefile, and the architecture inventory must agree.

## Authorized test changes

Design commits may restate the pathway spec blocks, source/entity tests needed
for missing proof, fixture routes, receipts, and live/routine registry entries.
Mechanical construction fixes may land with implementation while behavioral
assertions remain unchanged.

The src line ceiling may rise by at most 120 lines.
