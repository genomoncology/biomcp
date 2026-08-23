---
flow: build
priority: 31
---
# Make the variant-filter contract test reliable under load

`json_error_contract::contradictory_variant_filters_fail_before_myvariant_contact`
timed out after 10 seconds during the 2026-08-23 routine `make test` review.
The test launches `biomcp --json search variant` with contradictory filters and
expects the pre-dispatch validation error without provider contact. The review
run passed 3,017 of 3,018 Rust tests before this timeout.

The existing issue
`sdlc/issues/2026-08-23-contradictory-variant-filter-timeout.md` records this
as a watch item to file if it recurs; this is that recurrence. A focused
rerun passed, but took 22 seconds for the four subprocess cases, so the
failure appears timing-sensitive under the parallel suite. Determine whether
startup or validation work is causing the delay, then make the contract
reliable without weakening its no-provider-contact guarantee.

## Why this is priority 9

It is blocking the channel, not just itself. Ticket 1047 completed every stage and refused at 05-verify on 2026-08-23 because this timeout failed the landing gate on work that has nothing to do with variant validation. Any ticket in this repository can be stopped the same way at its last stage, having spent a full attempt.

## What done looks like, observably

- The test passes reliably when the full suite runs in parallel, not only when rerun in isolation. Whatever makes it timing-sensitive is named — startup cost, shared contention, an over-tight limit — and addressed at that cause.
- The no-provider-contact guarantee keeps full strength. The point of the test is that contradictory filters are rejected before any provider is contacted; a fix that relaxes that, or that raises the timeout without saying why the work takes as long as it does, is not a fix.
- Raising a limit is an acceptable answer only alongside a measurement showing what the time is spent on and that the new limit has real headroom over it.

Closes `sdlc/issues/2026-08-23-contradictory-variant-filter-timeout.md`.
