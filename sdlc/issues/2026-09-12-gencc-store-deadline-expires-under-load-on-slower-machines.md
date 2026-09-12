# GenCC store deadline expires under full-suite load on slower machines

Observed 2026-09-12 while running the full `make test` gate for ticket 1163 on
yellow.local (4 cores, 15 GB RAM, Ubuntu 25.10), the first full-suite run of
this branch on hardware slower than the 16-core dev box.

`entities::gene::gencc::tests::subprocess_lease_defers_old_generation_cleanup_until_reader_exits`
failed with `Err(Deadline)` unwrapped in the `publish` helper
(`src/entities/gene/gencc/tests.rs:468`). The run cancelled early under the
configured fail-fast, stopping at 1,478 of 3,452 tests with this as the only
failure.

The deadline is a fixed two seconds of wall clock. `Store::open()` calls
`open_until(Instant::now() + Duration::from_secs(2))`
(`src/sources/gencc/store.rs:111`), and `ensure_deadline` guards every step of
the publish pipeline, not just lock acquisition. During the failing run,
neighboring GenCC tests in the same suite took 10 to 27 seconds each under
four-way test parallelism, so the two-second budget can expire from external
CPU and fsync contention alone.

A second full-suite run the same morning failed the same way at 1,457 of
3,452 tests with 1,455 passed. Both failures were GenCC tests sharing this
root cause:

- `crash_boundaries_preserve_one_complete_namespace_generation` panicked at
  `tests.rs:468` on `Err(Deadline)` in the `publish` helper.
- `cleanup_faults_retain_unowned_or_unfinished_entries_for_a_later_pass`
  panicked at `tests.rs:844`: the store deadline expired before its second
  cleanup pass, and the assertion then found the retained entry that the
  deferred pass should have removed.

Evidence it is load-dependent rather than a data or logic bug:

- The test passes solo on yellow, three consecutive runs, 0.034 s each.
- The test passes solo on the 16-core dev box, 0.26 s.
- No assertion about generations or leases failed; only the deadline did.

Worth considering: whether the deadline should bound only lock acquisition
instead of total publish duration, whether tests should scale or override the
budget via an environment variable, and whether the fail-fast cancellation
should be relaxed for known machine-sensitive tests. A full `make test` on a
4-core host cannot currently complete the gate while this budget is fixed.

This is not only a test-portability problem. `Store::open` runs in the
production refresh path, and the same budget exhaustion can abort a real
refresh on a loaded or slow-disk host. Preferred direction: bound lock
acquisition with the deadline and let filesystem work proceed once the lock
is held; add a separate operation deadline only if the product needs one.
Test-side deadline scaling would hide that production behavior.
