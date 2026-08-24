---
---

# Investigate intermittent variant-filter contract timeout

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
