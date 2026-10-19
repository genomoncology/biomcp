---
priority: 1
---

# 2282 — Align pathway recordings with observed replies

Status: OPEN.

Milestone: 0.9.4

## Outcome

Pathway recordings and response metadata reflect the retained exact-request primary replies. Real source-consumer failures receive a separate bounded defect; no invented fields, weakened checks or assumed clinical facts hide them.

Changes: `testdata/replay/responses/reactome/**`, `testdata/replay/responses/wikipathways/**`, `testdata/replay/manifest.json`, `CHANGELOG.md`, `sdlc/records/2282-pathway-recording-fidelity.md`.

## Evidence

The seven authorized requests in job 2281/1792449561439-4540 all returned HTTP 200 under truthful current product headers. Private experiment 2100-biomcp-2281-checker-request-fidelity retains exact requests, full responses, raw hashes, timestamps and source/patch identity. Two Reactome structures match. ContainedEvents changes item type and two MAPK search recordings expect totalResults absent in actual replies. The BRAF search recording has historical HTTP 500 while current exact request returns 200. BRAF and WikiPathways search lack historical Content-Type, so those comparisons remain unverified. The HTTP success alone is not replay qualification.

## Keeps

No new provider calls. Adopt retained exact-request observations only after verifying the current source client request and licensing. Preserve primary source values in lawful minimal public projections; raw full replies remain private. Record original/new provenance and observed headers honestly. Do not turn a deliberate error scenario into a successful fixture: identify every consumer and preserve its failure contract under an explicitly owned fixture if needed, or report a separate fix. No runtime, checker or golden changes in this ticket. If current real wire behavior requires one, stop with the exact failing consumer and proposed bounded outcome. No clinical inference; delegate any clinical judgment. Unrelated manifest lanes own other rows and query-page paths; merge only independent fields. No extra duplicate full payloads.

## Proof

Before code, restate the outcome and reproduce the mismatch using retained raw replies against current main through pm job. Verify exact requests, source values, body hashes and safe projection provenance. Run affected source/strict replay consumers and lint through pm job. A retained observed fixture comparison must become clean without weakening the checker. No full gate; push once after green and stop for fresh review.

Keep: experiments/2100-biomcp-2281-checker-request-fidelity — retain the primary seven replies and request/provenance evidence reused by this repair.
