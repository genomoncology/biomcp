---
priority: 1
---

# 2276 — Resolve the disease paging deadline stall

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 7a88ab1, accept

## Outcome

The disease exact-resolution deadline control completes reliably while proving cancellation of an unresponsive provider. Preserve the bounded paging and provider-failure behavior; repair the actual cause, whether fixture synchronization or runtime cancellation.

Changes: `src/entities/disease/resolution/paging_tests.rs`, `CHANGELOG.md`, `sdlc/tickets/2258-admit-the-0-9-4-release-candidate.md`.

## Evidence

Actual main 17eb1e637c7a5883e70946c7f1db58ac738bbc06 passed the shared landing gate 2264/1792444403854-9054, including exact_resolution_deadline_cancels_a_provider_that_never_responds in 0.009 seconds. Actual-main full check 2156/1792444862466-82172 remains active with that exact Rust test alone still running. At observation its process 19074 had elapsed more than two minutes with no CPU use. No failure or root cause is yet established. Do not label main green or terminate a test merely for duration.

## Keeps

Inspect the existing stalled process and captured output before any reproduction. Distinguish a deterministic synchronization or cancellation defect from slow progress. Reuse the current main build artifacts without mutating its active checkout. No broad refactor, sleeps, retry-until-green or arbitrary timeout masking. Preserve production provider deadline semantics and complete paging guards. If no supported bounded repair emerges, report the blocker rather than inventing one.

## Proof

Record the actual cause and one controlled failing proof if a defect is established. Run affected paging and cancellation controls and necessary lint through pm job, then stop for fresh review. No full gate; the coordinator owns the final actual-main check. Retain diagnostics in a numbered private experiment folder with OWNER and Keep.

## Progress

- 2026-10-19 started

## Repair evidence

Developer c759ae17aad7abf2bfc3449bf2ad472fbc8df4a4 changes only the deadline fixture and its changelog bullet. The fixture receives the real provider request before pausing time; racing waits against resolver completion prevents stranded fixture acceptance. Virtual elapsed time still proves the intended provider deadline, and provider EOF proves cancellation closed the connection. Production paging and deadlines remain unchanged.

The original parked stack and listener-only socket diagnostics support the paused-clock fixture cause. The coordinator stopped verified stranded test process 19074 so original main check 2156/1792444862466-82172 ended incomplete: 4278 Rust cases passed, one deliberately terminated case failed, 1569 Python cases passed. This is not a duration failure. Raw original check remains in 2080/main-checks/1792444862466-82172.

First controlled injection 2276/1792445342961-42006 passed because the socket was already accept-ready; it is not reproduction evidence. Final controlled ordering 1792445431749-46416 failed an explicit early-completion invariant and establishes allowed completion-before-accept ordering, not its frequency or a deterministic untouched-test reproduction. Green 1792445478992-49744 passed all thirty-six resolution cases and make lint; the repaired control finished in 0.008 seconds. Published base c6c14dd64 plus private patch matched the final pushed source before commit. No full gate ran. PM approved the bounded synchronization repair and final-admission dependency.

Keep: experiments/808-biomcp-2276-deadline-fixture — retain process diagnostics, source patch comparison, both controlled ordering attempts, green proof and honest limitations.
