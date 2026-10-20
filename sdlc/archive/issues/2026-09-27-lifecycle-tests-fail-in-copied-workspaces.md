# Lifecycle tests fail in copied workspaces

Disposition (2026-10-20): Closed as moot because the affected lifecycle modules were deleted at e7e45eba5. Ticket 2243 recorded their residual closure at 569d9beb1.


Tracked 2026-09-27 from the review follow-ups ticket 1259. The
spec-contracts issue first called these failures pre-existing; the
2026-09-27 review asked for a tracking issue instead of an
uncorrected claim.

## Observation

`tests/test_routine_fixture_recovery.py` and
`tests/test_disease_survival_fixture_lifecycle.py` fail when run in
a copied workspace whose fixture scripts are only partially
populated — a failure shape verified by stashing on the base tree
and recorded in
`sdlc/archive/issues/2026-09-24-spec-contracts-omits-two-provider-fixtures.md`.
The 1254 batch-3 guard (skip the variant-identity fixture start
when its script is absent) removed the 127-exit shape this change
introduced; the remaining base-tree failures are this issue's
subject.

## What is wanted

A copied or partially provisioned workspace should get a clear
skip for every absent fixture, not a failing lifecycle test with a
stack trace that reads like a product fault.

## Status

Open. Owner: the 0.9.1 follow-up batch (tickets 1255-1259). Revisit
trigger: any new fixture script added to `scripts/run-specs.sh`
without the existence guard, or the next lifecycle-test failure in
a scratch checkout.

## Residual closure in 2243

Resolved: the two named lifecycle modules were deleted in e7e45eba5. No retained contract depends on their copied-workspace setup.

## Release dispositions (2026-10-19)

- Both copied-workspace lifecycle failures: MOOT. The two named modules were removed at e7e45eba5; 2243 at 569d9beb1 confirms no retained contract depends on them. No claim is made that their historical setup was repaired.
