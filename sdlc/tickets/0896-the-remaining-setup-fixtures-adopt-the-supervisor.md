---
flow: build
priority: 6
deps: ["0686", "0951", "0957"]
---
# The remaining setup fixtures adopt the supervisor

Ticket 0686 landed the owner-identity-aware supervisor for disease-survival.
This ticket reuses that implementation for twelve setup fixtures: five
remaining routine ownership helpers and seven direct setup fixtures.

## Exact scope

Routine helpers:

- article-fulltext-source
- ctgov-intervention-alias
- variant-identity
- clingen-cspec
- cpic

Direct setup fixtures:

- complexportal
- drug-ae-fallback
- mychem-empty
- section-outcomes
- study-download-error
- vaers
- article-federated-timeout

File-only fixtures are excluded.

## Done when

Each fixture launches through one generalized supervisor, exports the same
versioned owner record, and is reaped after normal exit, SIGTERM, SIGKILL,
timeout, stale-owner recovery, and PID reuse. Root-prefix, worktree, owner
token, process start identity, and process-group checks remain at least as
strict as 0686.

The current helper is hard-coded to disease marker names and recover-disease.
Generalize fixture kind and recovery dispatch without adding a second
supervisor or weakening path/identity validation.

## Authorized test changes

Design commits may restate tests/test_routine_fixture_recovery.py, the twelve
named setup/cleanup scripts, shared supervisor tests, and runner setup calls.
Earlier construction/lifecycle assertions may be reused, but every fixture
gets an owner-death test through its real exported-owner path.

The src line ceiling may not rise.
