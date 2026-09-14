
Root cause captured 2026-09-14 with RUST_BACKTRACE=1 on the gate host: the
abort is a Rust stack overflow, not a signal from the environment —
`thread 'biomcp-cli-execute' has overflowed its stack / fatal runtime error:
stack overflow, aborting`. The thread is spawned with an explicit 8 MiB
stack (`run_outcome_with_worker_stack`, src/cli/outcome.rs:632-648), the
same budget as a default main thread, and the identical code and toolchain
pass on the dev host. An 8 MiB overflow suggests deep or unbounded
recursion on a path that only runs on the gate host — plausibly
environment-triggered (Ubuntu 25.10 runtime, or a provider-policy lookup
that walks a symlinked PATH element differently). Next step for whoever
takes it: reproduce under gdb on the gate host, catch the guard-page fault,
and read the recursion; then either bound the recursion or make the
triggering branch host-independent. Until this lands, the gate host cannot
complete its own Rust lane (fail-fast cancels at this test), which is the
one remaining reason any Rust verification runs on the dev host.
