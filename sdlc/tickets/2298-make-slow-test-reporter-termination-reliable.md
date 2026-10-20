---
priority: 1
---

# 2298 — Make slow-test reporter termination reliable

Status: OPEN.

Milestone: 0.9.4

## Outcome

The slow-test reporter reliably forwards termination and reaps its runner, and its outside-in test observes the actual lifecycle without a readiness race. Restore main qualification with a bounded correction. Preserve routine slow-test warnings and failing test assertions.

Changes: `tools/report-slow-tests.py`, `tests/test_slow_test_reporting.py`, `sdlc/records/2298-slow-test-reporter-termination.md`, `CHANGELOG.md`.

## Current main failure

Actual main d1ced4374c45285683c02e5c6632426ae81db096 full job2297/1792519818443-70753 failed test_nextest_reporter_forwards_termination_and_reaps_runner in tests/test_slow_test_reporting.py. subprocess.communicate raised TimeoutExpired after three seconds for the reporter wrapping a child that prints its PID and sleeps20seconds. The full Python result was1657passed,1failed,11skipped. The preceding landing gate passed this test. The exact original log/state/result remains in experiment2102-biomcp-2258-final-live-replay/provider-policy-identities/2297/jobs/1792519818443-70753.

## Proof and stop rule

Before production edits, restate the outcome, reproduce on this current main and name the precise signal/readiness/reaping cause and changed files. Use a controlled real subprocess outside-in failure for the cause; do not rely on random repeated full-suite runs. Make the failing case pass and run only affected reporter checks through pm job. Distinguish a product lifecycle defect from a faulty test assumption. Do not blindly lengthen the deadline, weaken termination assertions, add sleeps or grow a new framework. Keep one repair lane; a material design choice goes to the PM through the coordinator. No full suite during development, no new public calls and no unrelated changes. Push once when review-ready. The coordinator runs reviewed repair landing and actual-main full qualification before resuming queued tickets.

## Bounded diagnosis and decision

Unchanged current-main focused job2298/1792520186484-55296 passes the existing lifecycle test in0.06seconds. Controlled real-subprocess job2298/1792520237130-63073 pauses the reporter after the forwarded runner PID, sends SIGTERM and observes the same three-second communicate timeout; after resuming execution it verifies exit143 and absent/reaped runner. This proves a caller deadline can reject eventual correct behavior, not that scheduling caused the original failure. Handler installation precedes forwarded PID, so the suspected readiness race is unsupported.

The full-main and attribution jobs overlapped for about71seconds. An earlier log sample shows load9.57/8.83/8.15, but the actual failure has no contemporaneous timestamp/load sample. Original cause remains unknown. No source defect was reproduced; no source file changed. The developer stopped after the agreed one controlled diagnostic.

PM decision needed: recommend one unchanged-main full retry after the focused pass, with the original failure and uncertainty retained; withdraw the speculative repair if qualification passes. Alternatively authorize a bounded event-based termination test replacement that preserves detection of missing forwarding/reaping. Do not infer an original cause or lengthen the deadline blindly.
