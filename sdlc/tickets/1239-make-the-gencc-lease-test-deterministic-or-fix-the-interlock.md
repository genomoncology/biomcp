# Make the GenCC lease test deterministic or fix the lease interlock

Filed from `sdlc/issues/2026-09-23-gencc-lease-test-flakes-under-full-suite-load.md`.
Before 0.9.1 per the review instruction.

## Problem

`entities::gene::gencc::tests::subprocess_lease_defers_old_generation_cleanup_until_reader_exits`
fails in full `make test` runs with `left: 2, right: 3` at
`src/entities/gene/gencc/tests.rs:923` — the generations directory holds
two entries where three are expected after publishing g3 with the child
holding g1's lease. Three occurrences in two days (tickets 1232, 1233,
and 1237 gates), each passing in isolation immediately after. The
cleanup path does take an exclusive `lease.lock` and skips
`WouldBlock` (`src/sources/gencc/store.rs:600-615`), so a plain missing
interlock is not the explanation; the failure needs reproduction under
load, not a guess.

## Design

1. Reproduce on the gate host: run the full suite (or the gencc test
   binary under nextest with the suite) repeatedly until the assertion
   fires, with the store's cleanup/lease tracing enabled.
2. From the trace, decide which of two fixes applies:
   - If a prune legitimately may prune g2 while g1 is leased (retention
     counting leased generations differently than the test assumes), the
     test's expectation is wrong and becomes a deterministic assertion
     of the actual policy.
   - If a prune deleted a leased or retained generation through a race
     (for example a quarantined `.delete-*` rename completing across
     publishes), fix the interlock in `store.rs` and keep the test.
3. Poll-with-deadline replaces any single-shot directory count read that
   can legitimately lag; the deadline is bounded with forced cleanup, per
   the 1236 review's waiting rule.

## Acceptance

- The failure is reproduced or the policy discrepancy is demonstrated
  with the trace cited in the record.
- The test passes in three consecutive full-suite runs on the gate host,
  or the underlying interlock fix does.
- A record lands; the issue file gains a Resolved section.

## Review

- Design review: pending
- Code review: pending
