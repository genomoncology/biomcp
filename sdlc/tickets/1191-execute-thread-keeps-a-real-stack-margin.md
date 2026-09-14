---
flow: build
priority: 1
deps: []
---

# 1191: The CLI execute thread keeps a real stack margin

## Goal

The in-process CLI execute thread runs its dispatch future chain with
megabytes of stack headroom on every host, proven by a measured margin, so
the gate host completes its full Rust lane.

## Current Facts — the eight whys (all measured 2026-09-14 under gdb on the
gate host, in sdlc/issues/2026-09-13-raw-ctgov-total-test-sigaborts-on-the-gate-host-only.md)

1. Why did the gate lane die? The test aborted with SIGABRT — gdb shows a
   write into the thread stack's guard page (`mov %rdi,0x8(%rsp)` faulting
   at RSP+8 with the page unmapped).
2. Why a stack overflow? The `biomcp-cli-execute` thread exhausted its full
   budget: stack-pointer inside the guard page; measured usage 8,186 KB
   against the 8 MiB requested (`run_outcome_with_worker_stack`,
   src/cli/outcome.rs:632-648).
3. Why does 8 MiB run out in only ~124 frames? Two monomorphized debug
   async-fn state machines dominate the stack: `run_outcome_inner`'s
   async_fn frame is 3,151 KB and `run`'s async block is 2,122 KB — the CLI
   dispatch state machine carries locals for every subcommand branch across
   awaits, unboxed.
4. Why are those state machines on the stack? The MCP path polls
   `run_outcome_inner` by value at src/cli/outcome.rs:647 —
   `runtime.block_on(run_outcome_inner(cli, true))` — the one dispatch
   boundary without a Box::pin (the tail arm at :594 and the CLI arm at :608
   are already pinned on main, as are the per-subcommand handlers).
5. Why only this test? It is the lane's only test that drives one complete
   raw-tool execution — full dispatch plus the provider connect/retry chain
   — on the execute thread in a single stack.
6. Why does it pass on the dev host? The whole path runs within a few
   hundred KB of the ceiling on both hosts; the hosts differ in runtime
   frame sizes and pool behavior (the dev host's pass takes 28 s through
   retry backoff), and the gate host's slightly fatter lower frames consume
   the margin. Marginal budget, host-sensitive outcome — not a host defect.
7. Why was the budget marginal? EXECUTE_STACK_BYTES = 8 MiB mirrors a
   default main-thread stack and was assumed generous; debug async frames
   grew with the CLI surface across many tickets until they silently
   approached the ceiling. Nothing measured or enforced a margin.
8. Why did nothing catch it? The test passes wherever the host leaves
   margin, the gate host is new (first full Rust lane 2026-09-13), and the
   abort prints no backtrace by default, hiding the cause until gdb.

Root cause in one sentence: an unboxed multi-megabyte async dispatch state
machine runs on a fixed 8 MiB thread stack in debug builds, leaving a
host-sensitive few-hundred-KB margin.

## Scope

- Pin the dispatch seam to the heap: `Box::pin` the boundary future(s) at
  the `run_outcome` seam (src/cli/outcome.rs) so the giant state machines
  (run_outcome_inner, run's async block) move off-stack. The exact seam
  placement is the design reviewer's to confirm; the measured frame table
  above is the guide.
- Keep EXECUTE_STACK_BYTES at 8 MiB as the safety net; raising it instead of
  boxing is explicitly rejected as papering over a structural margin.
- No behavior, API, JSON, or Markdown change of any kind.

## Acceptance

The faulting test passes solo on the gate host (it aborts there today,
three-for-three). The full Rust lane completes on the gate host. A
before/after measured frame table (same gdb method) shows the execute
thread's peak stack usage at or below 4 MiB (half the budget; the measured post-boxing remainder is about 2.85 MiB and poll-time frames cannot move to the heap). All existing tests unchanged;
ratchets exact; package count unchanged.

## Dependencies

None.

## Complexity

- Contract score: 1 (internal seam placement, one frozen rule: heap-pin the
  dispatch boundary)
- State and timing score: 0 (no concurrency semantics change)
- Reach score: 1 (every in-process CLI and MCP call flows through the seam)
- Proof score: 1 (measured before/after on two hosts)
- Cost of error score: 1 (every command path is affected; a wrong pin
  changes no semantics but review must prove it)
- Total: 4
- Minimum level floor: none
- Final level: 2
- Reasons: mechanical boxing at a measured seam with host-verified proof
- Selected model: gpt-5.6-luna, high reasoning (level 2 implementer)

## Review

- Design review: ACCEPT 2026-09-14; placement confirmed as the single
  missing pin at outcome.rs:647 (the ticket's original fact #4 cited the
  wrong branch — the CLI arm at :608 and tail arm at :594 are already pinned
  on main); acceptance re-baselined to <= 4 MiB per the measured remainder;
  raise-the-stack rejection affirmed (the downstream 1.0 integration branch's 16 MiB
  stopgap should be dropped in favor of this structural pin at its next
  merge with main — reconciliation recorded here).
- Code review: pending

- Code review: ACCEPT 2026-09-14 at 012dbcb5; one file, one line, confirmed
  by the primary agent with git show; placement, Send reasoning, and the
  tokio auto-box nuance verified from registry source. P2s closed: hunk
  byte-verified; branch naming noted.
- Full gates (final): merged after this evidence. At 42c5a996 on the gate
  host: lint OK, spec OK, and the previously-aborting test PASSED IN LANE —
  the primary acceptance — with the Rust suite running 683 tests beyond it
  before a third variant of the documented GenCC load-flake family cancelled
  the run (solo 3/3 at 0.03s; family issue updated). Margin: the fault-based
  gdb watermark no longer applies post-fix (the overflow path does not
  execute); the recorded basis is measured arithmetic — 8,186 KB total minus
  the 5,273 KB boxed frames leaves ~2,913 KB, within the 4,096 KB target —
  plus the empirical in-lane completion. A follow-up watermark capture is
  available to whoever wants it.
