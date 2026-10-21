---
flow: quickfix
priority: 6
---
# Validate trial search inputs before doing any provider work

## Done when

`biomcp --json search trial -c melanoma --lat=1 --lon=2 --distance=0`
returns an invalid-argument error naming `--distance`, without reaching
ClinicalTrials.gov. The same holds for out-of-range `--limit`,
`--offset` and `--next-page`, which are checked before the client is
constructed rather than after.

## Why here, why now

Two issues, one behaviour: bad input should be refused before any
provider work happens. They are merged because splitting them would put
two agents in the same validation path.

## The finding

Raised under March and carried over when BioMCP moved to the sdlc
factory. Reproduced in full below; `severity` is March's word, and
this ticket's priority is the one that counts.

<!-- from 595-reject-zero-trial-distance-before-provider-work.md -->

# Reject zero trial distance before provider work

## Summary

`biomcp --json search trial -c melanoma --lat=1 --lon=2 --distance=0` reaches ClinicalTrials.gov and returns an `api` error instead of rejecting the non-positive radius as client input.

## Detail

Ticket 595's numeric-validation exercise confirmed latitude and longitude now fail locally, but the adjacent `u32` distance accepts zero. `validate_location` checks only whether latitude, longitude, and distance are supplied together. A zero-mile radius is not useful and can be rejected by the provider, which misattributes bad client input as a registry outage. This predates ticket 595 and is outside its listed age/latitude/longitude scope.

## Suggested action

Validate `--distance` as positive at the shared trial filter boundary before alias/client work. Add a routine `spec/entity/trial-numeric-filters.md` scenario asserting exit 2, `invalid_argument`, and a `--distance` message anchor; add a native boundary test that accepts 1 and rejects 0. Intended improved-test destination: `spec` plus `test`.

<!-- from 595-validate-ctgov-pagination-before-client-construction.md -->

# Validate CTGov pagination before client construction

## Summary

The public CTGov `search_page` path constructs `ClinicalTrialsClient` before its
injected-client helper validates `--limit`, `--offset`, and `--next-page`.

## Detail

`src/entities/trial/search/mod.rs::search_page` now validates trial filters before
client construction, but `validate_search_page_args` remains inside
`search_page_with_ctgov_client`. Invalid pagination therefore performs avoidable
client setup before returning `invalid_argument`. This predates ticket 595 and does
not bypass that ticket's numeric guards, so it was not changed during review.

## Suggested action

Call the existing pagination validator at the public entity boundary before
`ClinicalTrialsClient::new`, retaining defensive validation in the injected-client
helper. Add a focused native ordering regression; do not alter shipped behavior.
