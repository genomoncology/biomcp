# Make the live verify lane resilient to transient Monarch and related-paper outages

Severity: nice-to-have.

This is live-verify-lane work. `make verify` is deliberately not a
gate rung here (`sdlc/planning/verify-lane.md`), so this cannot fail
a flight and should not outrank gate-lane work when it is triaged.

Carried over from March, where it was raised against ticket 606
on 2026-07-22 and left open. The text
below is as filed.
## Summary

`make verify` could not complete on 2026-07-22 because the live Monarch phenotype canary returned an HTTP middleware failure and the live related-paper citation canary exited nonzero. These are outside ticket 606's provider-query behavior.

## Detail

The ticket's real-provider strict-query canary passed independently with the release binary after the failure. The full verify command nevertheless exited 2 on these unrelated live assertions, leaving an operator unable to distinguish ticket evidence from transient upstream availability.

## Suggested action

Investigate the two provider failures and either repair the live integrations or classify expected upstream unavailability through the existing `verify-group`/operator policy. Add a `verify-group` test or harness check that preserves red failures for unexpected response shapes while reporting known transient availability explicitly.
