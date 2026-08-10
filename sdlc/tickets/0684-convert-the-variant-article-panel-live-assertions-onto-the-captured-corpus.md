---
flow: build
priority: 9
deps: ["0683", "0914", "0915", "0926"]
---
# Convert the variant-article panel live assertions onto the captured corpus

## Done when

The seven-variant panel uses 0683's real receipted corpus in the routine lane.
For every panel member, the fixture routes the exact production RequestPlan,
passes the recorded bytes through production decoders and orchestration, and
pins route attribution, identity evidence, recall fields, terminal state,
work accounting, and compact CLI rendering.

## Guardrails

- Strict routing rejects an unknown request. It never serves the nearest
  fixture.
- One real receipt per provider route anchors shape. Synthetic fixtures remain
  only for deterministic error edges and are labeled.
- The panel asserts BioMCP behavior, not that a provider is reachable today.
- Any retained public smoke is bounded, credential-aware, and stays in the
  live verify lane.
- This ticket owns variant-articles-live.md. Superseded 0885 does not.

Existing source RequestPlan and decoder tests remain; add only missing
consumed-plan, orchestration, process, and renderer proof. No identity
threshold, provider query, or work-budget change may be made to satisfy the
corpus.

## Authorized test changes

Design commits may restate variant-articles-live.md, its permanent routine
replacement, seven-panel fixture scripts, receipts, registry entries, and
tests that assert the same behavior. Mechanical construction fixes may land
with implementation without weakening assertions.

The src line ceiling may rise by at most 160 lines.
