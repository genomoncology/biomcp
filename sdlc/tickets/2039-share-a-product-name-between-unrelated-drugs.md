# 2039 — share a product name between unrelated drugs

Status: OPEN.

Milestone: 0.9.3

## Outcome

`get drug` with a product name that several drugs share never returns one drug's identifiers fused with another's targets or label. The card names what the record itself says, or refuses.

## Evidence

Filed 2026-10-10 from ticket 2038 (the pre-tag review), carrying 2031's deferral and 2035 finding 20: `get drug "Pain Relief"` gives a card carrying acetaminophen's DB00316 with TRPV1 targets through `hit_all_names` (`src/transform/drug.rs`); the leading-name fallback can admit another drug's synonym such as Terfenadine carboxylate.

- Starts from: the 2031 landing (merge 548f0b854).
- Keeps: the name, brand and synonym admission rule.
- Changes: decide identity from the record that owns the name, not from the merged name pool; refuse when the pool cannot name one drug.
- Proof: recorded-reply tests for the shared-name cases; a live sweep of shared product names.
- Defers: nothing.
