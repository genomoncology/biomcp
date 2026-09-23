# Close the panic recovery gaps

Filed from `sdlc/issues/2026-09-23-panic-recovery-gaps-after-1230.md`. Blocks 0.9.1.

## Design

1. **EMA and WHO byte stepping**: `src/sources/ema.rs:984` and
   `src/sources/who_pq.rs:717` advance `search_from = start + 1`; a term
   starting with a multibyte character (β-blocker) can slice inside a
   character when the word-boundary check fails. Advance by the first
   character's `len_utf8()` or iterate `match_indices`. Add a regression
   test with that input, and tests for the four slices ticket 1230 fixed,
   which have none.
2. **A real end-to-end panic test**: add a hidden test-only command that
   panics. Drive the spec binary over MCP stdio: the panicking call
   returns an `isError` result, then a normal call on the same session
   succeeds. This replaces the thread-join copy at
   `src/cli/outcome/tests.rs:4-23`, which cannot fail if abort returns.
3. **Recovery outside the execute seam**: `variant_normalize_car`,
   `variant_erepo`, `gene_cspec`, and `variant_articles` call entity code
   directly on the server runtime (`src/mcp/shell.rs:1116-1300`), as do
   `prepare_mcp_chart`, `outcome_to_mcp_output`, and redaction. Wrap every
   tool handler future in `catch_unwind` and map a panic to a tool error
   that includes the panic message, so no client waits forever.
4. **Margin on the unwind build**: rerun the deep trial and drug smoke
   commands on yellow against the new unwind release binary and record the
   result, closing the stale abort-build provenance in the 1230 record
   (this ticket's record notes the correction).
5. **Minor**: the join error includes the panic message
   (`outcome.rs:617`); poisoned locks in the DDInter index cache
   (`ddinter.rs:216`) and GenCC leases (`gencc/store.rs:899`) recover with
   `PoisonError::into_inner`.

## Acceptance

- The β-blocker input and the four 1230 slices have regression tests.
- The MCP stdio panic-survival spec test passes on yellow in debug and
  release profiles.
- A panic in any typed tool handler returns a tool error with the panic
  message and leaves the session usable, proven by tests.
- The unwind-build deep-command results are recorded.
- Full yellow gate at the head SHA; the issue file gains a Resolved
  section; a record lands.

## Review

- Design review: pending
- Code review: pending
