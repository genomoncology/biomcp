---
flow: build
priority: 1
---

# 2165 — Isolate the credential-redaction command contract

Status: OPEN.

Milestone: 0.9.3

## Outcome

Coordinator implementation brief: the real annotation CLI must complete its local credential-redaction contract without unrelated randomized retry delays. Preserve all credential, source, error and action assertions and the ten-second watchdog. Idle-connection probes did not establish the suspected serial-fixture cause; do not change concurrency on that basis. Recorded request timestamps instead show four LitVar 503 attempts followed by four PubTator 503 attempts, consuming most of the watchdog in randomized middleware backoff. Use a permanent local fixture failure for this redaction-only case, and require one request per provider. Establish the new request-count regression on current main before changing the fixture. Retry behavior remains covered by its existing dedicated cases.

The existing outside-in credential-redaction case reliably exercises its intended local provider failures without unrelated randomized retry delays.

- Starts from: `tests/json_error_contract.rs` and its existing subprocess/fixture helpers.
- Keeps: every credential-redaction assertion, structured errors, request behavior and hang protection. Do not replace the contract with a mock or increase its timeout without establishing the actual cause.
- Changes: claims `tests/json_error_contract.rs` `CHANGELOG.md`. Report any shared-helper expansion before implementation.
- Proof: reproduce the failure on current main with its exact command and record the cause before code. The unchanged test `swallowed_source_failures_do_not_log_credentials` failed during routine job `1792241265725-20923` on reviewed recovery 88834f90f after 4336 other tests passed. It timed out after 10 seconds running `biomcp --no-cache variant articles 'BRAF V600E' --strategy annotation --limit 1`, at line 47, with empty stdout and stderr. Investigate the local fixture and environment boundaries; add the smallest outside-in failing regression that establishes the cause, then run affected cases through the build job. The complete routine gate and actual-main check remain required.
- Defers: a broad fixture identifier/scenario refactor to 2163. Routine elapsed-time warning policy belongs to 2095.

## Reproduction and focused proof

Timestamp diagnostic `1792242091088-25783` established four LitVar transient attempts followed by four PubTator transient attempts, with middleware backoff consuming most of the command watchdog. Idle-connection probes passed and did not establish the initial concurrency hypothesis.

Current-main controlled job `1792242142979-48448` on `a1d96019d` replayed only the request-count regression in `swallowed_source_failures_do_not_log_credentials` through `cargo nextest run --locked --no-default-features --test json_error_contract -E 'test(swallowed_source_failures_do_not_log_credentials)'`. It exited 100 after 9.63 seconds, asserting LitVar request count 4 instead of 1. The complete injected test-only command is retained in that pm job.

Green job `1792242234211-29079` on `0ee990bc1` passed formatting, warnings-denied focused Clippy and all 23 cases in `json_error_contract`. The affected case took 0.076 seconds; the existing ten-second watchdog and every prior assertion remain unchanged. Only the two fixture status codes and the request-count assertion changed.
