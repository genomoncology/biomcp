---
flow: build
priority: 1
deps: []
---

# 1192: GenCC fixtures stop racing the clock

## Goal

The three GenCC fixtures that failed under full-lane load assert their
outcomes through generous-by-construction budgets or seam-level calls, so
sustained CPU saturation cannot change what they observe. Production
deadlines, budgets, and semantics do not change at all.

## Current Facts

Three documented load failures share one root-cause class
(sdlc/issues/2026-09-12-gencc-expired-budget-projection-test-starves-its-reserve-under-load.md):

1. `expired_refresh_budget_still_projects_authoritative_stale_data`
   (src/entities/gene/gencc/tests.rs:300) routes through
   `fetch_section(200ms)`. The production split reserves one quarter of the
   budget for projection (src/entities/gene/gencc.rs:92-94, capped at
   500ms), so acquisition burns 150ms spinning on the deliberately-held
   refresh lock (src/sources/gencc.rs:459-469) and the projection must
   finish inside the remaining 50ms of wall clock; a saturated core eats
   that window and `projection_unavailable` degrades the result to
   `(RefreshDeferred, Unavailable, 0)`. Deterministic repro: reserve to 1ms
   at gencc.rs:92 fails every run.

2. `post_rename_200_and_304_deadlines_return_committed_public_rows`
   (src/entities/gene/gencc/tests.rs:340) drives `fetch_section(2s)` with
   the `after-state-rename` fault injected. The recovery
   `committed_after_rename(authority_deadline)` re-opens the store with the
   SAME absolute 2s deadline (src/sources/gencc.rs:423-428 and its 304
   twin); when the fsync-heavy publish plus loopback HTTP consume the
   budget under load, the recovery's `Store::open_until` sees an expired
   deadline, returns `None`, and the command reports zero committed rows.

3. `cancelling_stalled_headers_and_streamed_body_drops_request_and_store_work`
   (src/sources/gencc.rs:865) gates on a 2-second wall-clock barrier for the
   spawned request to enter the fixture handler, then on 2-second bounds for
   provider-request drain and store settlement
   (src/sources/gencc.rs:834-860). Under lane load the whole test process
   can be descheduled past 2 seconds before the request even starts,
   tripping "request barrier: Elapsed" although nothing is wrong. The
   sibling `cancelling_active_publication_joins_cleanup_and_releases_locks`
   (src/sources/gencc.rs:924-975) carries the identical 2-second marker
   barrier and 5-second acquire budget and is fixed in the same stroke so
   the class leaves no member behind.

## Scope

Test-fixture changes only:

- Test 1 stops routing through `fetch_section` and asserts the seam it
  means to test: `GenCcClient::acquire_until` with a short acquisition
  deadline (the expired refresh budget) and a far-future authority
  deadline, then `project_until` with the same far-future deadline. The
  deferred-stale acquisition outcome and the `(RefreshDeferred, Stale, 3)`
  projection are asserted separately; the 50ms projection-window race no
  longer exists.
- Test 2 drives `acquire_until` directly with a 5-second acquisition
  deadline and a 30-second authority deadline, then projects with the same
  authority deadline. The fault-injection recovery keeps its full command
  assertions; the recovery store-open now has seconds of margin instead of
  whatever the publish left.
- Test 3 raises its barrier and drain bounds from 2 seconds to 60 seconds
  and the spawned acquire budget from 5 to 30 seconds, and applies the same
  raises to the sibling publication-cancellation fixture. The bounds remain
  regression tripwires — production's own internal deadline ends the
  request path long before them — but scheduler starvation cannot trip
  them.

Excluded: any change to production deadline constants, budget splits,
retry semantics, or error mapping; any change to other GenCC tests beyond
the named sibling barrier; new production code paths.
`src/sources/gencc.rs` stays at exactly 999 lines
(number-only edits); `src/entities/gene/gencc/tests.rs` stays below 1000.

## Acceptance

Each of the three tests passes deterministically while a CPU-saturating
helper (at least twice as many busy threads as cores) runs alongside: the
three tests and the full gencc family green under that induced load, plus
solo. The 1ms-reserve repro no longer fails test 1 (the projection window
is out of the test's path). No production file changes behavior; ratchets
exact; line counts within the stated bounds.

## Dependencies

None.

## Complexity

- Contract score: 1 (assertion seams change; equivalent coverage must be
  preserved per test)
- State and timing score: 1 (fixtures orchestrate locks, servers, and
  cancellation, but the change removes timing dependence rather than adds
  it)
- Reach score: 0 (one test family)
- Proof score: 1 (deterministic-under-load proof across three modes)
- Cost of error score: 0 (test-only; a wrong rewrite weakens one fixture)
- Total: 3
- Minimum level floor: none
- Final level: 2
- Reasons: fixture surgery at documented seams with a load-proof
  acceptance bar
- Selected model: gpt-5.6-luna, high reasoning (level 2 implementer)

## Review

- Design review: pending (parent-held evidence: mechanisms verified from
  source in this ticket's Current Facts)
- Code review: pending
