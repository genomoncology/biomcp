---
priority: 1
---

# 2176 — Repair the extra MCP query-validation request

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 819befe, accept

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
