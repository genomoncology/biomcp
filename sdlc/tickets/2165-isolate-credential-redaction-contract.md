---
flow: build
priority: 1
---

# 2165 — Isolate the credential-redaction command contract

Status: OPEN.

Milestone: 0.9.3

## Outcome

The existing outside-in credential-redaction case reliably exercises its intended local provider failures without ambient providers or configuration delaying the command.

- Starts from: `tests/json_error_contract.rs` and its existing subprocess/fixture helpers.
- Keeps: every credential-redaction assertion, structured errors, request behavior and hang protection. Do not replace the contract with a mock or increase its timeout without establishing the actual cause.
- Changes: claims `tests/json_error_contract.rs` `CHANGELOG.md`. Report any shared-helper expansion before implementation.
- Proof: reproduce the failure on current main with its exact command and record the cause before code. The unchanged test `swallowed_source_failures_do_not_log_credentials` failed during routine job `1792241265725-20923` on reviewed recovery 88834f90f after 4336 other tests passed. It timed out after 10 seconds running `biomcp --no-cache variant articles 'BRAF V600E' --strategy annotation --limit 1`, at line 47, with empty stdout and stderr. Investigate the local fixture and environment boundaries; add the smallest outside-in failing regression that establishes the cause, then run affected cases through the build job. The complete routine gate and actual-main check remain required.
- Defers: a broad fixture identifier/scenario refactor to 2163. Routine elapsed-time warning policy belongs to 2095.
