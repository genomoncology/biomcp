---
flow: build
priority: 1
deps: []
---

# 1187: The GenCC store deadline bounds only lock waits

## Goal

In `src/sources/gencc/store.rs`, a deadline passed to `open_until` bounds only
the wait to acquire locks. Filesystem work that begins after the relevant lock
is held no longer consumes that budget, so sustained host load cannot abort an
otherwise valid publish, cleanup, or read with `StoreError::Deadline`.

## Current Facts

The constructors store one `Instant` on the `Store`
(`open_until`, store.rs:148 region) and every method reuses `self.deadline`:
entry checks in `load` (253), `load_state` (270), `publish_cancellable` (393,
397, 411, 432, 449), mid-step checks in `load_generation` (330, 347),
`replace_state_locked` (483, 493, 501), the raw-temp checks
(`write_chunk`/`finish` 74-91, entry 189), and six bare
`Instant::now() >= self.deadline` comparisons in `cleanup_abandoned` (212, 233)
and `cleanup_locked` (508, 514, 533, 538). Two full `make test` runs on the
4-core gate host failed only through this shared budget: 1,477/1,478 and
1,455/1,457 tests passed, and the failures
(`subprocess_lease_defers_old_generation_cleanup_until_reader_exits`,
`crash_boundaries_preserve_one_complete_namespace_generation`,
`cleanup_faults_retain_unowned_or_unfinished_entries_for_a_later_pass`) all
expired the budget under load while neighboring fsync-heavy tests ran. Each
affected test passes solo in well under a second on the same host. Evidence and
analysis live in
`sdlc/issues/2026-09-12-gencc-store-deadline-expires-under-load-on-slower-machines.md`.

Production reaches the same path: `Store::open` itself is `#[cfg(test)]` with
a 2-second budget, but production calls
`Store::open_until(store_deadline(deadline))` at `gencc.rs` 158, 218, 414, and
433 with the operation deadline (120 seconds in `sync`). That stored deadline
bounds every post-open step and can abort a real refresh on a loaded host.

## Scope

Change, in `src/sources/gencc/store.rs` only:

- Lock waits keep the deadline: the store lock and anchor lock in
  `open_until`, and the generation-lease wait
  (`acquire_generation_lease_at` → `lock_shared_until(&lease, deadline)`,
  store.rs:854 region), which runs while the store lock is held and must stay
  bounded. Waiting past the deadline returns `StoreError::Deadline`.
- Remove every post-lock-hold budget check: the `ensure_deadline` entry and
  mid-step checks listed in Current Facts (189, 253, 270, 330, 347, 393, 397,
  411, 432, 449, 483, 493), the raw-temp checks (74-91, whose removal also
  deletes the orphaned `deadline` field of `RawCsvTemp`), and the six bare
  deadline comparisons in `cleanup_abandoned` and `cleanup_locked` so cleanup
  runs to completion under load instead of silently skipping work.
- One intentional exception stays: the post-rename budget check at
  `replace_state_locked` (store.rs:501) returning `StoreError::PostRenameSync`
  remains, with its recovery path and its test
  (`post_rename_200_and_304_deadlines_return_committed_public_rows`,
  src/entities/gene/gencc/tests.rs:340) unchanged. It has a safe, tested retry recovery, and removing
  it would require new fault-injection machinery outside this ticket's size.
- `StoreError::Deadline` narrows to: a lock wait timed out, or the publish was
  cancelled (the existing cancellation path in `publish_cancellable` returns
  `Deadline` and is untouched).

Exclude: per-operation deadlines for post-lock work (a later ticket if the
product needs one), any change to publish outputs or on-disk format, any
change to test-suite concurrency or fail-fast policy, and any new dependency.
The raw-temp checks are safe to remove because the outer operation deadline at
`gencc.rs` 358-367 already bounds the download.

## Tests

Deterministic, load-independent regressions:

1. Lock contention beyond the deadline. Existing coverage:
   `bootstrap_and_store_lock_waits_are_bounded_by_the_call_deadline`
   (`src/sources/gencc/tests.rs:629-674`) already proves `Deadline` on
   store-lock contention with a 30-millisecond budget. Extend the suite only
   with the missing half: after the holder releases, reopening the store
   succeeds and completes a publish.
2. Expired budget after open, split around the kept exception: with all
   locks free, a store opened with an already-expired deadline completes
   reads, and its publish fails with `StoreError::PostRenameSync` — never
   `Deadline` — which proves no removed check survived on the read and
   publish paths. A store with a live budget then completes deferred
   cleanup, and the assertions check cleanup effects concretely — the
   retained entry is actually removed and the generation count actually
   drops — not just that the calls return `Ok`.
   Amendment 2026-09-12: implementation proved the original wording
   (publish completes successfully on an expired budget) mutually exclusive
   with the kept post-rename exception, since `replace_state_locked` runs
   inside every publish and returns `PostRenameSync` on a past deadline
   deterministically. Confirmed by design review as the intended meaning;
   review note: cleanup and raw-temp paths are covered by the mechanical
   removal inventory and the existing cleanup suites, not by the expired
   half of this test.
3. The existing subprocess-lease deferral test passes unchanged.

## Acceptance

Tests 1-3 in the gencc test modules as fits the house layout. `make lint`,
`make test`, and `make spec` pass on the 4-core gate host from the pushed SHA.
The package path count stays exactly 1,300. Ticket 1163's full gates then
rerun on the same host from its updated branch.

## Complexity

- Contract score: 1 (crate-private error meaning narrows; several call sites
  and tests encode the current behavior)
- State and timing score: 2 (persistent store, cross-process locks and
  leases, subprocess tests, deferred cleanup)
- Reach score: 1 (one private module cluster and its tests; no public surface)
- Proof score: 1 (deterministic outcome-based tests; no hostile-input or byte
  proof)
- Cost of error score: 1 (production refresh could abort or wait wrongly;
  recoverable, retryable, no data loss)
- Total: 6
- Minimum level floor: level 3 (concurrency and shared durable state)
- Final level: 3
- Reasons: the concurrency floor sets the level; no public contract changes
- Selected model: gpt-5.6-sol, medium reasoning (level 3 implementer; level
  mapping per the workspace subagent-sdlc rubric: totals 6-8 are level 3)

## Review

- Design review: ACCEPT 2026-09-12 with findings; findings P1 (cleanup-path
  bare comparisons, post-rename decision), P2 (production budget wording,
  Deadline cancellation exception, lease-wait naming, test overlap)
  incorporated. Re-review: ACCEPT 2026-09-12; the four remaining citation
  corrections (lease-wait line, post-rename test line, 449 grouping, orphaned
  RawCsvTemp deadline field) are incorporated above. Implementable as written.
- Code review: ACCEPT 2026-09-12 at commit 8f0ed463; no blockers. Verified:
  removal inventory exact, three lock waits stay bounded (anchor, store,
  generation lease), cleanup runs to completion, cancellation and error
  identities preserved, tests match the amended acceptance list, no
  wrong-layer or smuggled changes. Report-only notes: `load()` on an
  expired budget with missing or invalid state.json can now surface
  PostRenameSync from the recovery path instead of Deadline (a mechanical,
  recoverable consequence of the kept checkpoint); rustfmt reflow enlarged
  the test diff cosmetically.
- Full gates: pending on the 4-core gate host
