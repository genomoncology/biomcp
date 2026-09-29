# Raw ctgov-total MCP test SIGABRTs on the gate host only

Observed 2026-09-13 during ticket 1164's gate run.

`mcp::shell::tests::ticket_1120::raw_biomcp_tool_preserves_an_omitted_ctgov_total_as_null`
aborts with SIGABRT at about 1.4 seconds on the 4-core gate host, solo and
deterministically, three consecutive runs, both from the routine-test archive
and from a fresh cargo build, at ticket 1164's tip and at unmodified main
(a637e359). The same test passes on the 16-core dev host at main. This is a
pre-existing gate-host-specific failure, not a regression from any current
branch.

Worth considering: the test spawns the raw biomcp tool as a subprocess; the
abort smells like another host-flavor interaction (the gate host runs Ubuntu
25.10 with uutils coreutils, which already produced the timeout defect
recorded 2026-09-12). Capturing the abort backtrace on the gate host and
minimizing the spawned-command difference between the two hosts is the next
step for whoever takes it.

Root cause captured 2026-09-14 with RUST_BACKTRACE=1 on the gate host:
the abort is a Rust stack overflow, not a signal from the environment —
`thread 'biomcp-cli-execute' has overflowed its stack / fatal runtime
error: stack overflow, aborting`. The thread is spawned with an explicit
8 MiB stack (`run_outcome_with_worker_stack`, src/cli/outcome.rs:632-648),
the same budget as a default main thread, and the identical code and
toolchain pass on the dev host. An 8 MiB overflow suggests deep or
unbounded recursion on a path that only runs on the gate host —
plausibly environment-triggered (Ubuntu 25.10 runtime, or a
provider-policy lookup that walks a symlinked PATH element differently).
Next step for whoever takes it: reproduce under gdb on the gate host,
catch the guard-page fault, and read the recursion; then either bound
the recursion or make the triggering branch host-independent. Until
this lands, the gate host cannot complete its own Rust lane (fail-fast
cancels at this test), which is the one remaining reason any Rust
verification runs on the dev host.

gdb method and raw frame table (2026-09-14, gate host, debug build at
main 1746c918, single faulting test under `gdb -batch` with `handle
SIGSEGV stop nopass`, frame sizes from consecutive stack-pointer deltas):

- Total frames at fault: 124. Total stack: 8,186 KB of the 8,192 KB
  budget.
- run_outcome_inner::{async_fn#0}: 3,151 KB (unpinned at the :647
  boundary).
- run::{async_fn#0}::{async_block#0}: 2,122 KB (already pinned at :594).
- with_no_cache::{async_fn#0}: 442 KB; Runtime::block_on: 438 KB;
  trial::dispatch::handle_search: 376 KB; long tail below.
- Faulting instruction: `mov %rdi,0x8(%rsp)` with RSP inside the guard
  page.

Ticket 1191 carries the fix: pin the :647 boundary; measured post-boxing
remainder is about 2.85 MiB.

Merged 2026-09-26 from the two "-only" files (typo and spelling
variants of this same investigation) by ticket 1254 batch 2; no
observation was dropped.

## Resolved

Root cause confirmed 2026-09-14 (8 MiB execute-thread stack overflow
in recursive XML walking) and fixed by ticket 1191's real stack
margin (backlog outcome commit `c716dc88`); the three duplicate
issue files were merged into this one by ticket 1254 batch 2
(2026-09-26). Ticket 1243's XML depth cap adds the mechanical guard
so the class cannot recur.
