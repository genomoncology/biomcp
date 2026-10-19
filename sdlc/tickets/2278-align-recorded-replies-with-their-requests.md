---
priority: 1
---

# 2278 — Align recorded replies with their requests

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 5a34545, accept

## Outcome

Five supported JSON recordings match the actual primary replies for their declared requests, with safe public projections and complete observed provenance. The source clients and current public contracts consume those replies truthfully.

Changes: `testdata/replay/responses/clinicaltrials/**`, `testdata/replay/responses/ctgov/**`, `testdata/replay/responses/mydisease/melanoma-detail.json`, `testdata/replay/responses/mydisease/mydisease-disease-mondo-0007947-bbfa1e3532db.json`, `tests/golden/**`, `testdata/replay/responses/pubmed/**`, `testdata/replay/manifest.json`, `CHANGELOG.md`.

## Evidence

Collection 2258/1792447971557-46385 on stable main 1c4753a453569d17dd66a13eb3d312ab8a5af4d4 retains exact requests, raw observed primary replies and structural differences in private M5 experiment 2096-biomcp-2258-live-collection. Four JSON entries drift: the NCT02576665 and NCT06131398 reference captures contain full records although declared requests project fewer fields (observed replies omit derivedSection, hasResults and other protocolSection modules); melanoma detail capture includes query-only _score absent in the detail response; one PubMed esummary response changes pmcrefcount types for 42550831 and 42772593. Do not infer data equivalence or a runtime defect before checking consumers.

## Keeps

Use the existing retained observed replies; do not repeat live requests. Verify the recorded request exactly matches current source client behavior. Re-record only lawful minimal public response data; raw replies stay private. Preserve original provenance and record the new observation honestly. Do not manually falsify expected provider data or weaken strict replay, drift checks, clinical identity or public goldens. If real current wire data breaks a source client or a product judgment is needed, report a separate bounded defect instead of changing behavior in this recording ticket. Manifest edits concern only these four recording rows and their response provenance; the parallel header lane owns unrelated missing Content-Types, and 2277 owns query-page paths. No denial or rate-limit reply becomes a successful fixture.

## Proof

Demonstrate the four existing structural mismatches using retained replies without new calls. Refreshed recordings pass the checker comparison against those exact observations and relevant strict replay/source consumers. Run affected tests and required lint through pm job only. No full gate; retain exact old/new hashes, request identity, private raw paths and limits. Stop if public redistribution rights or current wire behavior are not supported.

## Exact-request and golden disposition

Read-only investigation found observations 18 and 19 used unfiltered MyDisease detail URLs while current manifest keys and source client require a fields query. The coordinator authorized exactly two corrective primary requests; both succeeded as JSON and remain private in experiment 2098. The lane owns those two upstream declarations, observed Content-Types and capture provenance. No additional source calls are authorized for continuation. The four original structural mismatches and the additional exact-query recording now have supported observations.

Focused job 2278/1792448519432-89492 showed all five structural comparisons become clean, checker controls pass, and six public golden cases pass, but two CTGov references JSON goldens retain unrequested outcomes from the older full-record captures. The actual client succeeds on the exact projected request; requested references/summary, Markdown reference goldens and search-all goldens remain intact. The developer stopped without changing runtime or goldens. The PM explicitly approved updating only those two reference JSON goldens to truthful projected responses, preserving every requested-reference assertion and changing no runtime code. There is no product contract requiring unrequested outcomes.

Continuation adopts the exact retained recordings.patch (SHA256 a7073ed0ca8f6d1c9d8a9086ef405638a6eb8d4a1f1cd3ce798cc961785bf163), updates only the two identified JSON expectations and completes the previously unrun direct disease consumers, affected goldens and lint. Retain original red and new green independently. New provider calls, broader golden edits, runtime changes or inferred clinical facts are not authorized.

## Verified delivery

Developer stopped at f425138a61cba4fdb5ae642e49c8539b8d4edbac. The committed nine-file diff matches private checked patch SHA256 28440a1cc0fbaf6131b6ab6e51e116a191ed89fb518df5eb3216fbf8caf29379. Job 2278/1792448994029-42892 passed all eight affected strict goldens before an overly broad default-CLI assertion stopped that attempt. Job 2278/1792449104306-90818 passed both owned MyDisease exact-request consumers with source ID/name equality, five retained raw/hash/shape comparisons, lint and final patch parity. Unrelated strict-replay misses (3 and 17) remain retained and are outside the owned detail-query claim. No whole-command no-miss or release claim is made. Raw evidence, both failed and completed attempts, remains separately in experiment 2098-biomcp-2278-observed-capture-fidelity/continuation and continuation-final. The source-observed disease name is Marfan syndrome; the incorrect private probe label was corrected without clinical inference. No runtime changes or additional provider calls occurred.
