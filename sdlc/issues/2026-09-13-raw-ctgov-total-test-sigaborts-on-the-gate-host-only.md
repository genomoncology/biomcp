
gdb method and raw frame table (2026-09-14, gate host, debug build at main
1746c918, single faulting test under `gdb -batch` with `handle SIGSEGV stop
nopass`, frame sizes from consecutive stack-pointer deltas):

- Total frames at fault: 124. Total stack: 8,186 KB of the 8,192 KB budget.
- run_outcome_inner::{async_fn#0}: 3,151 KB (unpinned at the :647 boundary).
- run::{async_fn#0}::{async_block#0}: 2,122 KB (already pinned at :594).
- with_no_cache::{async_fn#0}: 442 KB; Runtime::block_on: 438 KB;
  trial::dispatch::handle_search: 376 KB; long tail below.
- Faulting instruction: `mov %rdi,0x8(%rsp)` with RSP inside the guard page.

Ticket 1191 carries the fix: pin the :647 boundary; measured post-boxing
remainder is about 2.85 MiB.
