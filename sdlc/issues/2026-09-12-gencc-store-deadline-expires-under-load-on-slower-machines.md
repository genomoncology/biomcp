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

Evidence it is load-dependent rather than a data or logic bug:

- The test passes solo on yellow, three consecutive runs, 0.034 s each.
- The test passes solo on the 16-core dev box, 0.26 s.
- No assertion about generations or leases failed; only the deadline did.

Worth considering: whether the deadline should bound only lock acquisition
instead of total publish duration, whether tests should scale or override the
budget via an environment variable, and whether the fail-fast cancellation
should be relaxed for known machine-sensitive tests. A full `make test` on a
4-core host cannot currently complete the gate while this budget is fixed.
