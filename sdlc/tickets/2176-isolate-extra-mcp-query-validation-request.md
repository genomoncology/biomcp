---
priority: 1
---

# 2176 — Repair the extra MCP query-validation request

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 819befe, accept

Reviews: revision f11f6dc, accept

## Outcome

The raw and typed article-query validation contract proves that invalid input makes no provider request, and ordinary valid prose has equivalent results and the correct explained request count. Name and fix the actual extra-request cause without weakening the invalid-input boundary or hiding retries.

- Evidence: configured landing gate 1792254419536-44879 on accepted startup repair 536db95c8 failed raw_and_typed_article_query_validation_converges_before_provider_work at tests/rmcp_client_contract.rs:1344, with three observed requests and two expected. All HTTP startup cases passed. Rust reported 4290 passes, one failure and 34 skips; Python reported 1166 passes and nine skips. No gate retry substitutes for finding the cause.
- Starts from: the existing RequestCounterFixture, request counter, raw and typed prose calls, and provider retry policy. Inspect before extending the scope.
- Changes: claims `tests/rmcp_client_contract.rs` `CHANGELOG.md`. A runtime cause or shared fixture correction requires an explicit report naming the additional file before editing it. Keep the fix local; the broader shared-scenario refactor remains deferred.
- Proof: reproduce the named cause through the real MCP/CLI contract on current source, recording requests or controlled concurrency as needed. Establish whether it is fixture leakage, an actual retry, or product behavior before changing an expectation. Preserve zero requests for invalid input and raw/typed result equality. Run only affected checks through the build job, obtain a fresh review, then gate the combined reviewed main repairs and run the full check on actual main.
- Keeps: refusal text and input redaction, no provider work for invalid queries, legitimate provider retry policy, isolated credentials/configuration/cache, and accepted startup diagnostics and slow-start behavior from 2175.
- Defers: broad fixture identifier refactoring, unrelated cache client changes, blind retrying or relaxing request counts.

## Investigation brief

Restated outcome at 4c4d75e5b: invalid queries make zero provider HTTP requests; raw and typed valid prose returns equal results with an explained request count. Initial inspection identifies the existing RequestCounterFixture in tests/rmcp_client_contract.rs: it counts accepted TCP connections before reading any request and ignores read length and errors. This is an inspection hypothesis, not yet a reproduced cause. Instrument the fixture and inspect provider retry behavior before changing its count. Keep implementation and the failing outside-in check in that test file, plus the changelog.

Focused investigation did not reproduce the extra request. Traced real MCP job 1792254795748-89178 passed all twenty iterations, each with four complete requests across prose and JSON pairs. Controlled job 1792254861597-94155 limited fixture reads to one byte before responding and closing; the MCP case still passed with its expected counts. Fragmented reads therefore do not establish the gate failure's cause. Diagnostic edits were removed, and no runtime or assertion changes were pushed. The next distinguishing evidence is request-line and transport-outcome tracing under the actual concurrent Rust workload.

## Diagnostic slice

Concurrent routine Rust job 1792255057254-3297 on the accepted startup repair plus request diagnostics passed 4335 tests with 34 skips and passed targeted compiler lint. The previously failing MCP case passed. Request count, response bytes, validation boundaries, result equality and retry behavior remain unchanged. The trace stores at most 32 fixed classifications and bounded numeric transport outcomes, without arbitrary query values, headers or error text. Fresh review accepted source b11c5c923 without changes or repeated checks. This is a diagnostic-only partial slice: the extra-request cause remains unknown and the original repair outcome is not complete. Keep the ticket open for recurrence while normal source qualification continues.

## Progress

- 2026-10-17 started
- 2026-10-17 landed 38610b46596bf9099e58203a2442c6e5714a0562; next: Safe request diagnostics landed in the shared2175slice after gate1792255317711-24775; actual-main check runs now. Original extra-request cause remains unknown; keep open for recurrence.
- 2026-10-17 next: PM permits closure after the final release rehearsal if no recurrence. Three clean full main checks since diagnostics landed:1792255511240-54847,1792256658840-2776,1792259305768-13311. Count subsequent checks after2172/2173 land too; original extra-request cause remains unconfirmed.
- 2026-10-17 next: Four clean full main checks since diagnostics landed; newest2156/1792260289066-54099 passed atbeb08a9. Include subsequent checks after2172/2173 and final rehearsal before closure; original cause remains unconfirmed.
- 2026-10-17 landed 22b4cecd292c8197e1ff9317e4f1d9926270d324; next: Reviewed counting-fixture cache-policy correction landed after gate1792262889556-22006. Run full main check and retain observation through final rehearsal; original extra-request cause remains unconfirmed.
- 2026-10-17 next: Fresh read-only cache-ownership inspection at087ff1d92: every test creates its own ContractHarness and owned TempDir; child environments isolate home/config/data/cache. Overrides remain owned by that test or its unique fulltext fixture. Query validation shares its cache only between raw/typed calls within that one test and keeps owners alive through shutdown. No ownership correction needed; original extra-request cause remains unconfirmed. Full main check1792263241297-93944 runs at22b4cecd2.
- 2026-10-17 next: Full main check2156/1792263241297-93944 passed at22b4cecd2 after the request-counting cache-policy correction. Four clean full checks preceded the stale-note recurrence and one clean full check follows its reviewed fix. Keep observing through cache changes and final rehearsal; original extra-request cause remains unconfirmed.
- 2026-10-17 next: Six full main checks passed since diagnostics: four before the stale-note recurrence and two after the reviewed no-store correction, latest54fb25e in2156/1792266562026-19174. Cache batch2172+2173 now landed62cefadf9 and its full check runs. Original extra-request cause remains unconfirmed; observe cache changes and final rehearsal.

## Recurrence after four clean full checks

Outcome restatement: Keep raw and typed validation equivalent, invalid input free of provider requests, and explain every valid request without suppressing genuine retry or stale-cache behavior. Name the newly observed reply difference before making a local repair.

Actual main a68a64a4fd81d3245bf700d53c7e39ae30790505 failed `make main-gate` in build job 1792262135795-41613. The outside-in `raw_and_typed_article_query_validation_converges_before_provider_work` failed at its full reply equality assertion: raw reply had no notes, typed reply added `Semantic Scholar data served from cache, 1 s old (older than the provider freshness window).` The diagnostic warning reported `age_seconds=0`. Rust passed 4305 cases before this failure and failfast; Python passed 1239 with nine skipped. No request-count assertion failed in this recurrence. Four prior full main checks passed. The non-recurrence closure condition is no longer satisfied; investigate this concrete boundary in `tests/rmcp_client_contract.rs` first and report any additional claimed file before editing it. Preserve the bounded request diagnostics and all existing refusal and request-count assertions.

Bounded reproduction plan: Exercise the existing complete raw/typed contract under supported `BIOMCP_CACHE_MODE=infinite`. Its counting fixture currently emits no freshness or non-storage policy. The first raw valid prose request returns a network reply; reuse can annotate the typed reply as stale. Demonstrate the unchanged full equality and count assertions failing before adding `Cache-Control: no-store` to this request-counting fixture. Then retain the explicit mode in the contract and require all original zero/two/four request counts and reply equality to pass. This tests fixture cache isolation without injected provider faults or a duplicate contract. Normal-mode revalidation and any historical extra retry remain separate from what this controlled case proves.

Deterministic RED before fixture correction: Job 1792262551130-97733 used published a68a64a4fd81d3245bf700d53c7e39ae30790505 plus only the existing contract setting `BIOMCP_CACHE_MODE=infinite`. Exact child command: `tools/with-build-identity cargo nextest run --locked --no-default-features --test rmcp_client_contract -E 'test(raw_and_typed_article_query_validation_converges_before_provider_work)' --no-capture`. It failed exit 100 with zero passed, one failed and 26 filtered at unchanged full-text equality: typed prose included the legitimate stale Semantic Scholar cache note, raw prose did not. Fixture replies are incorrectly storable for a contract whose zero/two/four counts require calls at the origin. Correct that counting fixture with `Cache-Control: no-store`, retain explicit cache reuse as the regression, and keep all reply, refusal and count assertions. This evidence does not establish the original three-versus-two cause or a transport-reset hypothesis.

Repair proof and disposition: Fresh review accepted f11f6dca464d2d06e8c1f42e02ba850e740274aa without changes. GREEN1792262648418-2645 passed formatting, both affected outside-in contracts (two passed, 25 skipped) and locked targeted Clippy with warnings denied. The counting fixture now returns `Cache-Control: no-store`; its explicit infinite-cache regression retains all full text/JSON equality, refusal/redaction, zero/two/four request counts and bounded diagnostics. No runtime retry or source behavior changed. Land this bounded correction as a partial slice immediately because actual main is red; other accepted product work waits for its full main check. Keep this ticket open through the final rehearsal under the PM recurrence ruling. The original extra-request cause and normal-mode revalidation transport trigger remain unconfirmed.
