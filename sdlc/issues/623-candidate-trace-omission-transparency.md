# Make bounded candidate-route trace omissions observable

Severity: should-fix.

Carried over from March, where it was raised against ticket 623
on 2026-07-29 and left open. The text
below is as filed.
## Summary

`candidate_trace` is capped at `ITEM_WORK_LIMIT`, but merged candidates can retain
more than that many route observations. The current debug schema states that it is
bounded without saying when observations were omitted.

## Detail

A later visible result can lack a trace row if earlier candidates have multiple
route observations. That makes the live canary conservatively fail its
route-attribution gate, but leaves an operator unable to distinguish a genuine
pipeline loss from diagnostic-trace truncation.

## Suggested action

Choose and document a deterministic trace selection policy (prefer visible and
filtered candidates) plus an omission indicator, or revise the trace contract to
make its sampling semantics explicit. Add a frozen fixture with more observations
than the bound and a behavioral `spec` assertion for the selected policy and
omission signal.
