# GenCC expired-budget projection test starves its 50 ms reserve under load

Analyzed 2026-09-12 after one load-dependent failure of
`entities::gene::gencc::tests::expired_refresh_budget_still_projects_authoritative_stale_data`
during a full `make test` run on the 4-core gate host (assertion at
`src/entities/gene/gencc/tests.rs:323`: expected `(RefreshDeferred, Stale, 3)`,
got `(RefreshDeferred, Unavailable, 0)`). The test passed solo three times on
the same host, in two other full runs of the same code, and on the 16-core
dev host.

Mechanism, verified from source: `fetch_section` splits its 200 ms budget into
a 150 ms acquisition window and a fixed one-quarter projection reserve
(`src/entities/gene/gencc.rs:92-94`). The test deliberately holds the refresh
lock, so acquisition consumes its full 150 ms spinning
(`src/sources/gencc.rs:459-469`), leaving about 50 ms for the projection.
`project_until`'s outer deadline then expires under four-way test load and
`projection_unavailable` (`src/entities/gene/gencc.rs:230-242`) degrades to
`(RefreshDeferred, Unavailable, 0)` — the designed degradation path, with the
authoritative generation intact on disk for the next query. The test and all
budget plumbing are byte-identical on pre-1187 main, so this predates the
store deadline change; nothing suggests data loss.

Worth considering: asserting `acquire_until`'s outcome directly in this test
instead of routing through the outer projection deadline, or enlarging the
projection reserve for this test. Deterministic repro for whoever takes it:
set the reserve locally to 1 ms at `gencc.rs:92` and the focused test fails
with exactly this tuple on every run and host; revert and it passes.

Second instance in the same family, 2026-09-13: during full-lane load,
`post_rename_200_and_304_deadlines_return_committed_public_rows` failed once
at tests.rs:401 (0 committed rows where 3 were expected) after an 11-second
runtime; it passes solo in about 0.03 seconds and had passed in every prior
full run since ticket 1187 rewired it to fault injection. Same shape as the
first instance: a GenCC fixture whose expectations shift when sustained load
stalls the store's synchronous work past the window the fixture assumes.
Worth considering alongside the first: both tests share the gencc_env serial
family, and both would benefit from asserting outcomes that do not depend on
wall-clock windows.
