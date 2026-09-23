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
3. **The end-to-end panic test targets the outer dispatch**: the
   injection is an environment-guarded test tool, not a cargo feature —
   `--all-features` release builds (the `Makefile` full-feature lane and
   the `spec/surface/build-profile.md:30-31` full-feature release proof)
   must stay unaffected, and a feature cannot be excluded from
   `--all-features`. When `BIOMCP_TEST_PANIC_TOOL=1` is set at server
   startup, one extra tool registers on the router after `catalog::apply`
   (`src/mcp/catalog.rs:12-56`), outside the catalog; it panics inside
   tool dispatch, behind the new `catch_unwind`. The variable is unset in
   every gate, release, and production path, so the artifact carries an
   inert registration branch and the public surface stays exactly the
   seven catalog tools. The exact value `"1"` alone enables it. The
   contract client removes the variable for every spawned child
   (`env_remove("BIOMCP_TEST_PANIC_TOOL")` before applying `extra_env` in
   `crates/biomcp-mcp-contract-client/src/lib.rs`), so the seven-tool
   contract test (`tests/rmcp_client_contract.rs:652-654`) cannot inherit
   a parent setting, and only the panic test adds the variable. The hook
   is documented as internal and narrowly scoped, following the precedent
   of the shipped test signal in `spec/README-timings.md:206-208`. The
   test drives the built binary over MCP stdio with the existing contract
   client (same-session calls per
   `tests/rmcp_client_contract.rs:1491-1496`): the panicking call returns
   an `isError` result, then a normal call on the same session succeeds.
   The old thread-join copy test is removed.
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
   (`:917-919`). All five sites recover through one shared helper that
   maps `PoisonError::into_inner` to the guarded value; the guarded
   values are reconstructible (DDInter caches a rebuildable index; GenCC
   stores weak lease references, revalidates file identity, prunes, and
   reinserts). The tests exercise the shared helper directly, so the
   recovery behavior is proven once for all sites; the non-Unix GenCC
   site is compiled out on the gate host, so its branch is covered by the
   helper's unit tests and named as a residual.
6. The join error includes the panic message (`outcome.rs:616-618`
   currently discards the payload through `map_err(|_| ...)`).

## Acceptance

- The β-blocker input and the four 1230 slices have regression tests.
- The MCP stdio panic test passes on yellow in debug and release profiles,
  proving `isError` then a same-session success at the dispatch level.
- A panic in any typed tool handler returns a tool error with the panic
  message and leaves the session usable.
- The unwind-build deep-command results are recorded.
- Every listed lock site calls the shared recovery helper, whose behavior
  is tested directly; the non-Unix GenCC call site remains compile-only
  residual on the Linux gate host.
- Full yellow gate at the head SHA; the issue file gains a Resolved
  section; a record lands.

## Review

- Design review: REJECT three times 2026-09-23 (gpt-5.6-sol, medium).
  First: the panic test must exercise the outer dispatch, the injection
  must be gated rather than `#[cfg(test)]` or a hidden command, and
  poison recovery covers all five acquisition sites. Second: a cargo
  feature would enter `--all-features` release builds and conflict with
  the seven-tool `catalog::apply` invariant — the injection became an
  environment-guarded registration outside the catalog, and the poison
  sites share one tested helper. Third: the contract client must
  `env_remove` the variable for every child, and the acceptance wording
  must state helper-tested recovery with the non-Unix GenCC site as
  compile-only residual. Fourth: REJECT on stale review bookkeeping only
  (the history said "third review pending"); fixed in the same commit.
  Fifth: ACCEPT 2026-09-23 — history accurate, design complete, no P2s.
- Code review: ACCEPT 2026-09-23 (gpt-5.6-sol, medium) — full-diff review;
  route stripped before catalog validation and restored after; count
  assertions pin every lock site to the helper
- Verification: yellow gate at f9c6839e lint/test/spec OK; release-profile
  stdio panic test OK; unwind-build margin rerun clean; see
  `sdlc/records/1235-close-the-panic-recovery-gaps.md`
