# 2341 — Keep broad variant search pages stable

Status: OPEN.

Milestone: 0.9.4

## Outcome

Broad variant search pages never repeat or skip a result across consecutive pages. BioMCP either pages in the same order the provider pages, or fetches enough to sort locally before applying the offset, and says which order it uses. Exact-identity search stays unchanged.

- Proof: an outside-in replay test whose provider returns stable IDs 3, 2, 1 shows limit-1 pages 1, 2 and 3 returning each ID exactly once; a larger replay case shows the union of pages equals one unpaged result with no duplicates.

## Evidence

- On 2026-10-21, the release runtime reviewer of integration 3c8aee87a found that broad variant pagination forwards the provider offset and then applies a different local sort. With stable provider IDs 3, 2, 1, consecutive limit-1 pages can repeat 1. An isolated loopback probe on Gator is authorized to confirm it. If the probe disproves the counterexample, withdraw this ticket with the probe as evidence.
