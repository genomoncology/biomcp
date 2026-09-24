# Make the GenCC lease test deterministic or fix the lease interlock

Filed from `sdlc/issues/2026-09-23-gencc-lease-test-flakes-under-full-suite-load.md`.
Before 0.9.1 per the review instruction.

## Problem

`entities::gene::gencc::tests::subprocess_lease_defers_old_generation_cleanup_until_reader_exits`
fails in full `make test` runs with `left: 2, right: 3` at
`src/entities/gene/gencc/tests.rs:923` — the generations directory holds
two entries where three are expected after publishing g3 with the child
holding g1's lease. Three occurrences in two days (tickets 1232, 1233,
and 1237 gates), each passing in isolation immediately after. The
cleanup path does take an exclusive `lease.lock` and skips
`WouldBlock` (`src/sources/gencc/store.rs:600-615`), so a plain missing
interlock is not the explanation; the failure needs reproduction under
load, not a guess.

## Design

Root cause (verified by review against `cleanup_locked`, store.rs:550-575):
the classification loop pushes ANY `load_generation` error into the
`invalid` list, and invalid generations are pruned without the
`newest_other` retention protection. Under full-suite pressure the load
of the healthy unleased g2 fails transiently, g2 is deleted, and the
directory holds {g1(leased), g3} = 2. The child's lease never mattered
to g2; classification is the sole deletion-decision seam (reviewer
confirmed every other error path retains or aborts).

Review decision (first review REJECT): `StoreError::Invalid` is what the
transient failures actually surface as — `open_existing_at` maps every
`fd < 0` to `Invalid`, and `read_at` maps read failures to `Invalid` —
so variant discrimination alone would not close the hole. Chosen fix:

1. Taxonomy fix in the helpers: a pure errno mapping
   (`store_error_for_errno`) returns `Unavailable` for environment
   errnos (EINTR, EIO, EMFILE, ENFILE, ENOMEM) and `Invalid` otherwise,
   with no errno reading as `Invalid`. Applied at exactly three sites on
   the load path: `open_existing_at`'s `fd < 0`, its `metadata()`
   failure, and `read_at`'s `read_to_end` failure. The deliberate
   validity checks (mode/uid/nlink, schema, digest) stay `Invalid`.
   Unit tests cover the pure mapping both directions.
2. Classification discrimination: only `Err(StoreError::Invalid)`
   enters the invalid list; `Unavailable`, `Deadline` (routine lease
   contention), `PostRenameSync`, and any future variant retain the
   generation this pass with `tracing::warn!(generation = %name, %error,
   "GenCC cleanup retained a generation after a transient load
   error")`. The next publish retries; retained generations are bounded
   by the publish rate while the environment error persists.
3. `tracing::warn!(generation = %name, ...)` also fires when an invalid
   generation is pruned, so deletions are no longer silent.
4. Deterministic regression test at the classification seam, in-process
   and serial like the existing FAIL_AT family: publish g1, g2; arm
   `BIOMCP_GENCC_TEST_FAIL_AT=cleanup-classify-generation` (returns
   `Unavailable` as the load result, debug-only); publish g3; assert the
   publish succeeds and the generations directory holds three entries;
   disarm; assert the store loads. On unfixed code the injection point
   does not exist, so the armed variable is inert and the count-3
   assertion fails against normal cleanup's 2; with the injection added
   but the old any-error classifier, every unleased non-active
   generation would prune to 1. The corrupt-prune policy stays pinned
   by the existing
   `invalid-finalized` test, which passes unchanged under the taxonomy
   fix.
5. The flaking test's count assertion stays single-shot: cleanup runs
   synchronously inside publish under the exclusive store lock.

The full-suite load reproduction is retained as evidence, not as the
design's dependency: the deterministic seam test plus the taxonomy unit
tests prove the mechanism either way.

## Acceptance

- The failure is reproduced or the policy discrepancy is demonstrated
  with the trace cited in the record.
- The test passes in three consecutive full-suite runs on the gate host,
  or the underlying interlock fix does.
- A record lands; the issue file gains a Resolved section.

## Review

- Design review: REJECT once (the fix's discrimination signal did not
  exist — transient open errors surface as `Invalid`); revised to the
  taxonomy fix above, ACCEPT on re-review 2026-09-24
- Code review: ACCEPT 2026-09-24 (reviewer, medium) — two P2s recorded
  below
- Verification: yellow gate at 6b09b548 lint/test/spec OK, plus two more
  full `make test` runs green at the merged SHA 6836ed54 — three
  consecutive full-suite passes
- Code review: pending

## Recorded residuals

- Non-unix `read_regular` still maps transient read failures to
  `Invalid`, so the same transient-prune hazard exists on non-unix
  hosts. Deferred with the design's P2; fix is routing it through the
  errno taxonomy.
- Live provider verification and the full-suite load reproduction stay
  with their owning tickets.
