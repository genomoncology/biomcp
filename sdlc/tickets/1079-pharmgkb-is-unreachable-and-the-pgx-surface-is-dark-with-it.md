---
flow: build
priority: 6
---

# PharmGKB is unreachable and the pgx surface is dark with it

Health check (2026-08-28): `PharmGKB | error | 118ms (connect)`. Affects "pgx
recommendations and annotations" — the entire `search pgx` command has no live
source behind it. A connect error at 118ms is an endpoint problem (moved URL,
stalled TLS, or DNS), not a key problem: no key is reported configured.

## Done when

- The PharmGKB integration connects again; root cause named in the ticket
  body (moved host, changed path, or protocol requirement).
- `search pgx` returns live recommendations and annotations, and `get drug`
  pgx sections populate.
- The health check reflects the fix, and a regression note in the health
  table documents what broke so the next endpoint move is diagnosable in
  one look.

Filed as build: the suite is green (make test / make lint exit 0,
2026-08-27); the failure is a live-upstream break, not a reproducible red in
the test lane.
