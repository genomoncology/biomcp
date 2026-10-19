---
priority: 1
---

# 2278 — Align recorded replies with their requests

Status: OPEN.

Milestone: 0.9.4

## Outcome

Four supported JSON recordings match the actual primary replies for their declared requests, with safe public projections and complete observed provenance. The source clients and current public contracts consume those replies truthfully.

Changes: `testdata/replay/responses/clinicaltrials/**`, `testdata/replay/responses/ctgov/**`, `testdata/replay/responses/mydisease/melanoma-detail.json`, `testdata/replay/responses/pubmed/**`, `testdata/replay/manifest.json`, `CHANGELOG.md`.

## Evidence

Collection 2258/1792447971557-46385 on stable main 1c4753a453569d17dd66a13eb3d312ab8a5af4d4 retains exact requests, raw observed primary replies and structural differences in private M5 experiment 2096-biomcp-2258-live-collection. Four JSON entries drift: the NCT02576665 and NCT06131398 reference captures contain full records although declared requests project fewer fields (observed replies omit derivedSection, hasResults and other protocolSection modules); melanoma detail capture includes query-only _score absent in the detail response; one PubMed esummary response changes pmcrefcount types for 42550831 and 42772593. Do not infer data equivalence or a runtime defect before checking consumers.

## Keeps

Use the existing retained observed replies; do not repeat live requests. Verify the recorded request exactly matches current source client behavior. Re-record only lawful minimal public response data; raw replies stay private. Preserve original provenance and record the new observation honestly. Do not manually falsify expected provider data or weaken strict replay, drift checks, clinical identity or public goldens. If real current wire data breaks a source client or a product judgment is needed, report a separate bounded defect instead of changing behavior in this recording ticket. Manifest edits concern only these four recording rows and their response provenance; the parallel header lane owns unrelated missing Content-Types, and 2277 owns query-page paths. No denial or rate-limit reply becomes a successful fixture.

## Proof

Demonstrate the four existing structural mismatches using retained replies without new calls. Refreshed recordings pass the checker comparison against those exact observations and relevant strict replay/source consumers. Run affected tests and required lint through pm job only. No full gate; retain exact old/new hashes, request identity, private raw paths and limits. Stop if public redistribution rights or current wire behavior are not supported.
