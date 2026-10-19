---
priority: 1
---

# 2276 — Resolve the disease paging deadline stall

Status: OPEN.

Milestone: 0.9.4

## Outcome

The disease exact-resolution deadline control completes reliably while proving cancellation of an unresponsive provider. Preserve the bounded paging and provider-failure behavior; repair the actual cause, whether fixture synchronization or runtime cancellation.

Changes: `src/entities/disease/resolution/paging_tests.rs`, `src/entities/disease/resolution/paging.rs`, `src/entities/disease/resolution.rs`.

## Evidence

Actual main 17eb1e637c7a5883e70946c7f1db58ac738bbc06 passed the shared landing gate 2264/1792444403854-9054, including exact_resolution_deadline_cancels_a_provider_that_never_responds in 0.009 seconds. Actual-main full check 2156/1792444862466-82172 remains active with that exact Rust test alone still running. At observation its process 19074 had elapsed more than two minutes with no CPU use. No failure or root cause is yet established. Do not label main green or terminate a test merely for duration.

## Keeps

Inspect the existing stalled process and captured output before any reproduction. Distinguish a deterministic synchronization or cancellation defect from slow progress. Reuse the current main build artifacts without mutating its active checkout. No broad refactor, sleeps, retry-until-green or arbitrary timeout masking. Preserve production provider deadline semantics and complete paging guards. If no supported bounded repair emerges, report the blocker rather than inventing one.

## Proof

Record the actual cause and one controlled failing proof if a defect is established. Run affected paging and cancellation controls and necessary lint through pm job, then stop for fresh review. No full gate; the coordinator owns the final actual-main check. Retain diagnostics in a numbered private experiment folder with OWNER and Keep.
