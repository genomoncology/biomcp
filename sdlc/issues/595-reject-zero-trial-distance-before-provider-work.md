# Reject zero trial distance before provider work

Severity: should-fix.

Carried over from March, where it was raised against ticket 595
on 2026-07-19 and left open. The text
below is as filed.
## Summary

`biomcp --json search trial -c melanoma --lat=1 --lon=2 --distance=0` reaches ClinicalTrials.gov and returns an `api` error instead of rejecting the non-positive radius as client input.

## Detail

Ticket 595's numeric-validation exercise confirmed latitude and longitude now fail locally, but the adjacent `u32` distance accepts zero. `validate_location` checks only whether latitude, longitude, and distance are supplied together. A zero-mile radius is not useful and can be rejected by the provider, which misattributes bad client input as a registry outage. This predates ticket 595 and is outside its listed age/latitude/longitude scope.

## Suggested action

Validate `--distance` as positive at the shared trial filter boundary before alias/client work. Add a routine `spec/entity/trial-numeric-filters.md` scenario asserting exit 2, `invalid_argument`, and a `--distance` message anchor; add a native boundary test that accepts 1 and rejects 0. Intended improved-test destination: `spec` plus `test`.
