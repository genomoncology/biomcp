# GenCC subprocess-lease test flakes under full-suite load

Filed 2026-09-23 after the second occurrence in a week. Not a release blocker.

## Symptom

`entities::gene::gencc::tests::subprocess_lease_defers_old_generation_cleanup_until_reader_exits`
fails in full `make test` runs on the gate host with:

```
assertion `left == right' failed
  left: 2
 right: 3
```

at `src/entities/gene/gencc/tests.rs:923`. Observed 2026-09-23 in the
ticket 1232 gate (08:5x) and the ticket 1233 gate (10:5x). It passes
three-for-three in isolation immediately after each failure, and the full
suite passes on rerun. The historical "GenCC load flakes" class was
recorded as resolved by PRs #273-#280; this is a distinct assertion.

## Cause

Untested hypothesis: the lease-count assertion at `:923` reads the
generation count after subprocess readers exit; under parallel suite load
the reader exit can lag the assertion window, leaving one generation
unaccounted (left 2, right 3).

## Fix

Reproduce under load (`cargo nextest run` with the suite, repeated),
then make the assertion deterministic: wait for the lease release or poll
with a deadline instead of reading the count once, mirroring the
determinization ticket 1228 applied to the cache expiry test.

## Priority

Should-fix after 0.9.1. Track here if a third occurrence lands.
