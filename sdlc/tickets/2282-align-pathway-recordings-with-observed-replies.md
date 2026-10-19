---
priority: 1
---

# 2282 — Align pathway recordings with observed replies

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 79ac5d7, accept

Reviews: revision 0604527, accept

## Outcome

Pathway recordings and response metadata reflect the retained exact-request primary replies. Real source-consumer failures receive a separate bounded defect; no invented fields, weakened checks or assumed clinical facts hide them.

Changes: `testdata/replay/responses/reactome/**`, `testdata/replay/responses/wikipathways/**`, `testdata/replay/manifest.json`, `CHANGELOG.md`, `sdlc/records/2282-pathway-recording-fidelity.md`, `tests/fixtures/pathway/synthetic-reactome-failure/body.json`, `tests/golden/test_search_all_provider_failure.py`, `tests/golden/cases/search-all-braf-melanoma.json`, `tests/golden/cases/search-all-braf-melanoma.md`, `tests/golden/cases/search-all-braf-melanoma-counts-only.json`, `tests/golden/cases/search-all-braf-melanoma-counts-only.md`, `tests/golden/cases/search-all-braf-melanoma-synthetic-reactome-failure.json`, `tests/golden/cases/search-all-braf-melanoma-synthetic-reactome-failure.md`, `tests/golden/cases/search-all-braf-melanoma-counts-only-synthetic-reactome-failure.json`, `tests/golden/cases/search-all-braf-melanoma-counts-only-synthetic-reactome-failure.md`.

## Evidence

The seven authorized requests in job 2281/1792449561439-4540 all returned HTTP 200 under truthful current product headers. Private experiment 2100-biomcp-2281-checker-request-fidelity retains exact requests, full responses, raw hashes, timestamps and source/patch identity. Two Reactome structures match. ContainedEvents changes item type and two MAPK search recordings expect totalResults absent in actual replies. The BRAF search recording has historical HTTP 500 while current exact request returns 200. BRAF and WikiPathways search lack historical Content-Type, so those comparisons remain unverified. The HTTP success alone is not replay qualification.

## Keeps

No new provider calls. Adopt retained exact-request observations only after verifying the current source client request and licensing. Preserve primary source values in lawful minimal public projections; raw full replies remain private. Record original/new provenance and observed headers honestly. Do not turn a deliberate error scenario into a successful fixture: identify every consumer and preserve its failure contract under an explicitly owned fixture if needed, or report a separate fix. No runtime, checker or golden changes in this ticket. If current real wire behavior requires one, stop with the exact failing consumer and proposed bounded outcome. No clinical inference; delegate any clinical judgment. Unrelated manifest lanes own other rows and query-page paths; merge only independent fields. No extra duplicate full payloads.

## Proof

Before code, restate the outcome and reproduce the mismatch using retained raw replies against current main through pm job. Verify exact requests, source values, body hashes and safe projection provenance. Run affected source/strict replay consumers and lint through pm job. A retained observed fixture comparison must become clean without weakening the checker. No full gate; push once after green and stop for fresh review.

Keep: experiments/2100-biomcp-2281-checker-request-fidelity — retain the primary seven replies and request/provenance evidence reused by this repair.

## PM disposition and verified delivery

The PM approved normal BRAF pathway search using the real retained 200 response. The historical provider-failure behavior moves to an explicitly named synthetic scenario, preserving its four original assertions. The four normal search-all goldens gain success-path assertions for observed pathway rows. No runtime or checker changes are authorized. Synthetic failure is excluded from live drift comparison through the existing separate replay-root control. This ruling expands the original claim only to the named failure-scenario test/fixture and affected snapshots.

Developer stopped at 5a5a90280d035d88dec2257466c5bcba3902f63b. Checked patch SHA256 6a9ba86f06f639df9021d097e91ce83b034b1f56cb72dcfc1c4486b6a10f5f19 matches the committed implementation diff against published stable main 1c4753a453569d17dd66a13eb3d312ab8a5af4d4. Inspection 2282/1792449744175-3788 reproduced retained-reply drift. Development 1792449864318-55982 demonstrated four controlled old-snapshot failures then passed 20 replay and 16 source cases. Final comparison 1792450044221-55907 passed all seven source-value/header/structure comparisons, preserved original failure snapshots byte for byte, and passed those behavior tests; formatting failed. Mechanical job 1792450093186-70747 applied Ruff, proved AST identity and golden hash parity, and passed full repository lint and patch parity without another behavior suite. Every attempt remains retained; no provider calls or full gate occurred. Historical missing headers remain unknown, and public projections sample consumed fields.

Keep: experiments/2101-biomcp-2282-pathway-recording-fidelity — retain source comparisons, exact private patches, controlled failures, formatting attempts and completed qualification evidence.
