# Close the panic recovery gaps

Filed from `sdlc/issues/2026-09-23-panic-recovery-gaps-after-1230.md`; revised after a REJECT design review (2026-09-23). Blocks 0.9.1.

## Design

1. **EMA and WHO byte stepping**: `src/sources/ema.rs:984` and
   `src/sources/who_pq.rs:717` advance `search_from = start + 1` after a
   rejected word-boundary match; for a term starting with a multibyte
   character (`β-blocker` inside `β-blockers`) the next slice panics.
   Advance by the first character's `len_utf8()` — the smallest
   behavior-preserving fix (`match_indices` changes overlapping-match
   iteration). Add a regression test with that input, and tests for the
   four slices ticket 1230 fixed.
2. **Recovery at the tool dispatch**: rmcp runs each request in a bare
   spawned task (`rmcp-1.7.0/src/service.rs:963-979`), and the direct
   handlers (`variant_normalize_car`, `variant_erepo`, `gene_cspec`,
   `variant_articles` at `src/mcp/shell.rs:1115-1339`), chart preparation,
   and output conversion (`src/cli/outcome.rs:651-654`) bypass the worker
   seam. Wrap the generated `ServerHandler::call_tool` override around
   `self.tool_router.call(...)` in one place with
   `FutureExt::catch_unwind(AssertUnwindSafe(...))`, converting `String`,
   `&str`, and fallback payloads into a sanitized `CallToolResult::error`
   that includes the panic message. The server stores only the router
   (`shell.rs:29-32`); no handler keeps server-owned mutable guards across
   dispatch.
3. **The end-to-end panic test targets the outer dispatch**: a
   feature-gated test-only tool (`#[cfg(feature = "panic-test")]`),
   registered only when the feature is on — not a hidden clap command,
   which would stay publicly accepted — injects a panic inside tool
   dispatch, behind the new `catch_unwind`, not inside the worker join.
   The test drives the built binary over MCP stdio with the existing
   contract client (`crates/biomcp-mcp-contract-client/src/lib.rs:56-64`,
   same-session calls per `tests/rmcp_client_contract.rs:1491-1496`): the
   panicking call returns an `isError` result, then a normal call on the
   same session succeeds. The feature appears in no default or release
   build, and the public catalog stays at seven tools
   (`src/mcp/catalog.rs:10-48`). The old thread-join copy test is removed.
4. **Margin on the unwind build**: rerun the deep trial and drug smoke
   commands on yellow against the unwind release binary and record the
   result; the shipped profile stays pinned by the Python profile pin
   (`tests/test_upstream_planning_analysis_docs.py:1181-1189`), which
   remains the durable guard, because a cargo test child alone is not
   proof of the shipped profile.
5. **Poison recovery at every acquisition site**: the DDInter index cache
   at `src/sources/ddinter.rs:216`, its lookup path (`:209-211`), and its
   eviction path (`:225-228`); the GenCC lease store at
   `src/sources/gencc/store.rs:899` and its second, non-Unix acquisition
   (`:917-919`). All recover with `PoisonError::into_inner`; the guarded
   values are reconstructible (DDInter caches a rebuildable index; GenCC
   stores weak lease references, revalidates file identity, prunes, and
   reinserts).
6. The join error includes the panic message (`outcome.rs:616-618`
   currently discards the payload through `map_err(|_| ...)`).

## Acceptance

- The β-blocker input and the four 1230 slices have regression tests.
- The MCP stdio panic test passes on yellow in debug and release profiles,
  proving `isError` then a same-session success at the dispatch level.
- A panic in any typed tool handler returns a tool error with the panic
  message and leaves the session usable.
- The unwind-build deep-command results are recorded.
- Every listed lock site recovers from poison, proven by tests.
- Full yellow gate at the head SHA; the issue file gains a Resolved
  section; a record lands.

## Review

- Design review: REJECT once 2026-09-23 (gpt-5.6-sol, medium) — three
  findings folded in: the panic test must exercise the outer dispatch
  (the worker join already recovers), the injection must be a
  feature-gated tool rather than `#[cfg(test)]` or a hidden command, and
  poison recovery covers all acquisition sites, not only the two first
  cited. Second review pending.
- Code review: pending
