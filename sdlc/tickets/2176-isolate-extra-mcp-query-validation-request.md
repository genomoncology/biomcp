---
priority: 1
---

# 2176 — Repair the extra MCP query-validation request

Status: OPEN.

Milestone: 0.9.3

## Outcome

The raw and typed article-query validation contract proves that invalid input makes no provider request, and ordinary valid prose has equivalent results and the correct explained request count. Name and fix the actual extra-request cause without weakening the invalid-input boundary or hiding retries.

- Evidence: configured landing gate 1792254419536-44879 on accepted startup repair 536db95c8 failed raw_and_typed_article_query_validation_converges_before_provider_work at tests/rmcp_client_contract.rs:1344, with three observed requests and two expected. All HTTP startup cases passed. Rust reported 4290 passes, one failure and 34 skips; Python reported 1166 passes and nine skips. No gate retry substitutes for finding the cause.
- Starts from: the existing ArticleSearchFixture, request counter, raw and typed prose calls, and provider retry policy. Inspect before extending the scope.
- Changes: claims `tests/rmcp_client_contract.rs` `CHANGELOG.md`. A runtime cause or shared fixture correction requires an explicit report naming the additional file before editing it. Keep the fix local; the broader shared-scenario refactor remains deferred.
- Proof: reproduce the named cause through the real MCP/CLI contract on current source, recording requests or controlled concurrency as needed. Establish whether it is fixture leakage, an actual retry, or product behavior before changing an expectation. Preserve zero requests for invalid input and raw/typed result equality. Run only affected checks through the build job, obtain a fresh review, then gate the combined reviewed main repairs and run the full check on actual main.
- Keeps: refusal text and input redaction, no provider work for invalid queries, legitimate provider retry policy, isolated credentials/configuration/cache, and accepted startup diagnostics and slow-start behavior from 2175.
- Defers: broad fixture identifier refactoring, unrelated cache client changes, blind retrying or relaxing request counts.
