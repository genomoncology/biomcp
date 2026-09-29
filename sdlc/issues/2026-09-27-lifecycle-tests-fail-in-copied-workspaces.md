# Lifecycle tests fail in copied workspaces

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
`sdlc/issues/2026-09-24-spec-contracts-omits-two-provider-fixtures.md`.
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
