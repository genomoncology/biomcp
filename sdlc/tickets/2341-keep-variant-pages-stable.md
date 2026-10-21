# 2341 — Keep broad variant search pages stable

Status: OPEN.

Milestone: 0.9.4

## Outcome

Broad variant search pages never repeat or skip a result across consecutive pages. BioMCP either pages in the same order the provider pages, or fetches enough to sort locally before applying the offset, and says which order it uses. Exact-identity search stays unchanged.

- Changes: `src/entities/variant/search/mod.rs`, `tests/test_variant_search_pagination.py`, `tests/feature-links.json`, `tests/golden/cases/search-variant-braf-melanoma-limit3.json`, `tests/golden/cases/search-variant-braf-melanoma-limit3.md`, `docs/user-guide/variant.md`, `CHANGELOG.md`.

- Proof: an outside-in replay test whose provider returns stable IDs 3, 2, 1 shows limit-1 pages 1, 2 and 3 returning each ID exactly once; a larger replay case shows the union of pages equals one unpaged result with no duplicates.

## Evidence

- Coordinator restatement: Consecutive broad variant pages must preserve one stable provider order so page unions match the unpaged result without repeats or omissions. Exact-identity precision filtering stays on its existing separate path.
- Reproduction and planned proof: Fresh reviewer Gator j2686 reproduced the actual CLI defect at 3c8aee87a69c5745412e46ef706b9f11c7c209cc: limit-one offsets zero, one and two each returned chr1:g.1A>T, while limit three returned IDs one, two and three. The authored loopback provider received from=0/size=40, from=1/size=40, from=2/size=40 and from=0/size=120. No live provider was called. Raw probe, command, fixture, job log and manifest are retained on Yellow under data/biomcp/ticket-evidence/2330/review-pagination; log SHA256 0ef129259206014541c0aafd27906a97347ad382af62afb28a6fb5a28895402a. j2686 exit zero means the defect was reproduced, not product correctness.
- Planned files and scope: src/entities/variant/search/mod.rs, tests/test_variant_search_pagination.py, tests/feature-links.json, docs/user-guide/variant.md and CHANGELOG.md. Outside-in controls require every stable provider ID exactly once across limit-one pages and a larger page union.

- On 2026-10-21, the release runtime reviewer of integration 3c8aee87a found that broad variant pagination forwards the provider offset and then applies a different local sort. With stable provider IDs 3, 2, 1, consecutive limit-1 pages can repeat 1. An isolated loopback probe on Gator is authorized to confirm it. If the probe disproves the counterexample, withdraw this ticket with the probe as evidence.
