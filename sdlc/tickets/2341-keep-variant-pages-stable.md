# 2341 — Keep broad variant search pages stable

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 8e5fc9940f58373a0551adad05149937c876ff75, accept

Reviews: revision 488c8169cb519b118540be582f280bb4b8522317, accept

Landed: 94a02bf

## Outcome

Broad variant search pages never repeat or skip a result across consecutive pages. BioMCP either pages in the same order the provider pages, or fetches enough to sort locally before applying the offset, and says which order it uses. Exact-identity search stays unchanged.

- Changes: `src/entities/variant/search/mod.rs`, `tests/test_variant_search_pagination.py`, `tests/feature-links.json`, `tests/golden/cases/search-variant-braf-melanoma-limit3.{json,md}`, `tests/golden/cases/search-all-braf-melanoma*.{json,md}`, `docs/user-guide/variant.md`, `CHANGELOG.md`.

- Proof: an outside-in replay test whose provider returns stable IDs 3, 2, 1 shows limit-1 pages 1, 2 and 3 returning each ID exactly once; a larger replay case shows the union of pages equals one unpaged result with no duplicates.

## Evidence

- Coordinator restatement: Consecutive broad variant pages must preserve one stable provider order so page unions match the unpaged result without repeats or omissions. Exact-identity precision filtering stays on its existing separate path.
- Reproduction and planned proof: Fresh reviewer Gator j2686 reproduced the actual CLI defect at 3c8aee87a69c5745412e46ef706b9f11c7c209cc: limit-one offsets zero, one and two each returned chr1:g.1A>T, while limit three returned IDs one, two and three. The authored loopback provider received from=0/size=40, from=1/size=40, from=2/size=40 and from=0/size=120. No live provider was called. Raw probe, command, fixture, job log and manifest are retained on Yellow under data/biomcp/ticket-evidence/2330/review-pagination; log SHA256 0ef129259206014541c0aafd27906a97347ad382af62afb28a6fb5a28895402a. j2686 exit zero means the defect was reproduced, not product correctness.
- Planned files and scope: src/entities/variant/search/mod.rs, tests/test_variant_search_pagination.py, tests/feature-links.json, docs/user-guide/variant.md and CHANGELOG.md. Outside-in controls require every stable provider ID exactly once across limit-one pages and a larger page union.

- On 2026-10-21, the release runtime reviewer of integration 3c8aee87a found that broad variant pagination forwards the provider offset and then applies a different local sort. With stable provider IDs 3, 2, 1, consecutive limit-1 pages can repeat 1. An isolated loopback probe on Gator is authorized to confirm it. If the probe disproves the counterexample, withdraw this ticket with the probe as evidence.

- Broad searches now request the caller's exact provider window and preserve provider ordering, filters and total counts. The separate exact-identity scan/filter/rank path is unchanged. Outside-in controls cover descending three-ID limit-one pages and a larger filtered nine-ID page union. Golden changes preserve captured values and provider replies, changing only resulting order and navigation suggestions; search-all retains its own significance ordering.
- Affected Gator j2737 passed both pagination controls, twenty exact-identity compatibility cases, six affected golden commands and formatting/lint. Earlier j2711 and j2727 failed because the new test used the wrong metadata envelope, not because they reproduced the product defect. Authoritative original RED remains the actual CLI probe j2686. Their failed statuses are preserved.
- Direct Gator make landing-gate j2739 passed exit zero at clean published 26e2ef7dc4b109a8320312877b9f2c1375a642ee. It completed lint, routine Rust/Python, strict docs and offline specifications. Yellow data/biomcp/ticket-evidence/2341 retains raw logs and the authored test; SHA256SUMS has SHA256 88bfff61de59dcb80d9f2de11b53def77427f3cda3b8cdf4d1379e627fb66403.
- Fresh reviewer accepts that exact source with no findings. The coordinator independently verified the terminal gate result. This Evidence-only update changes no gate input; land with the other accepted review corrections.
