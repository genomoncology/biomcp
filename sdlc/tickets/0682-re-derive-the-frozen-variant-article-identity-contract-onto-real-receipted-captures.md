---
flow: build
priority: 7
---
# Re-derive variant-article identity onto real receipted captures

## Done when

The frozen exact-variant article identity contract uses real receipted
provider observations for successful identity, route, passage, and CAR/CAID
claims. The routine CLI fixture passes those bytes through the production
decoders and identity orchestration instead of inventing rows, passages, or
opaque identifiers inline.

Keep explicit synthetic fixtures for degraded, malformed, timeout, and other
edge outcomes a real provider cannot reliably produce. Label them synthetic;
they supplement rather than replace the real anchors.

## Proof required

- Each real anchor is captured through the production RequestPlan and has a
  receipt binding request and response.
- A local executor fixture proves the production request is consumed.
- The production decoders, identity merge, route status, terminal state, and
  work accounting run over those bytes.
- Process-level JSON and Markdown assertions pin the identity fields and
  provenance without depending on public availability.
- Unknown or mismatched fixture routes fail rather than falling back to a
  convenient response.

Do not change identity thresholds, route priority, work budgets, or provider
semantics to fit a capture.

## Authorized test changes

Design commits may restate
spec/fixtures/run-variant-article-identity-fixture.sh, its owning spec blocks,
the identity fixture corpus/receipts, and relevant production-path tests.
Mechanical construction fixes may land with implementation while assertions
stay unchanged.

The src line ceiling may rise by at most 120 lines.
