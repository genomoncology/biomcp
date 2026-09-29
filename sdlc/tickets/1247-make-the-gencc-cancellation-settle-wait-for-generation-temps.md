# Make the GenCC cancellation settle wait for generation temporaries

Filed from the 2026-09-24 CI failure at main commit `2dceaae2` (the
1245 merge; the failure is unrelated to that merge's content). Same
family as ticket 1239's load findings.

## Problem

`sources::gencc::cancelling_active_publication_joins_cleanup_and_
releases_locks` asserts, once and immediately after
`assert_cancelled_store_settles`, that `generations/` holds no `.tmp-`
directories (src/sources/gencc.rs:979-984). The settle helper polls with
a deadline for store lock release, the active etag, and no `.raw-`
temporaries at the root — but the `.tmp-` check inside `generations/`
sits outside that loop. Under CI load, the aborted publication's
generation-temp cleanup can lag past the settle point, and the
single-shot read fails the suite (`assertion failed` at :979, observed
once in CI on 2026-09-24; passing on the gate host before and after).

## Design

Move the `.tmp-`-in-generations condition into
`assert_cancelled_store_settles`'s existing poll loop, beside the
`.raw-` check, so every settle criterion is waited on with the same
60-second deadline. The post-settle assertion at :979 then verifies
stability (a second read immediately after the loop) rather than doing
the waiting itself. No production code changes.

## Acceptance

- The settle helper polls for all three criteria (lock/etag, `.raw-`
  at root, `.tmp-` in generations).
- The renamed or kept assertion after the helper still catches a
  leaked temp deterministically.
- Yellow gate green; the mutation (dropping the poll condition) fails
  the test.

## Review

- Design review: combined review REJECT once (code motion, not
  waiting), rework ACCEPT 2026-09-24
- Verification: yellow gate at 17ac88cc lint/test/spec OK; see
  `sdlc/records/1247-make-the-gencc-cancellation-settle-wait-for-generation-temps.md`
- Code review: covered by the combined review with the design (REJECT
  once — the first version was code motion, not waiting — then ACCEPT
  after the rework) 2026-09-24
