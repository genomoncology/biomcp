---
base: 9f1fd34a
head: f9c6839e
---

Closed the panic recovery gaps from the issue filed 2026-09-23.

The EMA and WHO searches advance by the first character's `len_utf8()`
after a rejected word-boundary match (`ema.rs:984`, `who_pq.rs:717`), so a
term starting with a multibyte character such as β-blocker can no longer
slice inside a character. Regression tests cover that input and the four
slices ticket 1230 fixed. One `ServerHandler::call_tool` override wraps
router dispatch in `catch_unwind(AssertUnwindSafe(...))` and returns a
sanitized tool error carrying the panic message, so a panic in any typed
tool handler — including the four direct handlers, chart preparation,
and output conversion that bypass the worker seam — no longer leaves an
MCP client waiting forever. The old thread-join copy test is replaced by
a real end-to-end test: with `BIOMCP_TEST_PANIC_TOOL=1` exactly, a route
stripped before `catalog::apply` and restored after it panics inside
dispatch; over stdio the call returns `isError` with the injected
message and the next call on the same session succeeds. The contract
client removes the variable for every spawned child, and the hook is
documented as internal beside the timings precedent. The DDInter index
cache (three acquisitions) and the GenCC lease store (two) recover from
poisoned locks through one shared `recover_poison` helper
(`PoisonError::into_inner`), with count assertions pinning every site to
it. The execute-thread join error now includes the panic payload
message.

Evidence: code review ACCEPT (Sol medium) over the full diff; yellow gate
at f9c6839e — `make lint`, `make test`, `make spec` all OK, the
release-profile `cargo nextest run --release -E
'test(rmcp_stdio_recovers_from_tool_panic)'` OK, and the deep-command
margin rerun on the new unwind release binary (38,532,656 bytes): the
four #282 overflow commands and the JSON regulatory command all clean
(three exit 0 with results, `drug interactions apixaban` exit 1 with the
expected DDInter-unavailable error on this host), no stack overflow. The
package boundary is pinned at 1,351 with the new helper named.

Process note: both implementer runs died (a harness runner timeout after
a steer, then a runner startup timeout); the orchestrator finished and
verified the takeover — the diff was complete and faithful to the
accepted design when taken over, and the independent code review ran on
the final commit.

Residuals: the non-Unix GenCC acquisition is compile-only on the Linux
gate host, covered by the shared helper's unit tests. The DDInter leg of
the margin rerun reached only the clean source-unavailable error on
yellow; the run with real synced DDInter data on the M5 is the remaining
verification. The shipped profile stays pinned by the Python profile pin;
the cargo test child is not treated as proof of it.

Post-merge corrections on main: the CI formatter wanted two additional
shell.rs forms (import grouping, tool-attribute wrap) the gate host's
formatter accepted as written; the rust-source-size baseline for
`src/mcp/shell.rs` moved to 2,229 with a ticket-1235 authorization; and
the panic hook variable gained its production-read classification
(`BIOMCP_TEST_PANIC_TOOL`, internal test hook). CI is green at 0cc8b13e
on all five jobs, and the documentation publish converged. The
gate-host-versus-CI lane disagreement on identical code (formatter and
env-docs contract) is noted for the 1238 hygiene investigation.
