# Panic recovery gaps after ticket 1230

Filed 2026-09-23 from a review of ticket 1230 (merge fe7ebae5). Unwinding is back in every shipped profile, and the four slice fixes are correct. The Python pin at `tests/test_upstream_planning_analysis_docs.py:1180-1191` is the only guard against `panic = "abort"` returning. One reachable panic remains.

## A non-ASCII drug name still panics the EMA and WHO searches

`src/sources/ema.rs:984` and `src/sources/who_pq.rs:717` advance with `search_from = start + 1`. When the term starts with a multibyte character and a match fails the word-boundary check, `field[search_from..]` slices inside a character. A name like `β-blocker` against a field containing `β-blockers` triggers it.

Fix: advance by the first character's `len_utf8()`, or iterate `match_indices`. Add a regression test with that input. Add tests for the four slices 1230 fixed; none has one.

## The new test cannot fail if abort returns

`src/cli/outcome/tests.rs:4-23` copies the thread join rather than calling `run_outcome_with_worker_stack`. Cargo ignores the profile's panic setting for test builds, so the comment's claim is false.

Fix: add a hidden test-only command that panics. Drive the spec binary over MCP stdio: the panicking call returns an `isError` result, then a normal call on the same session succeeds.

## Several MCP paths sit outside the recovery point

Recovery is only the join at `src/cli/outcome.rs:600-619`. `variant_normalize_car`, `variant_erepo`, `gene_cspec`, and `variant_articles` call entity code directly on the server runtime (`src/mcp/shell.rs:1116-1300`), as do `prepare_mcp_chart`, `outcome_to_mcp_output`, and redaction (`shell.rs:360+`). rmcp runs each request in a bare `tokio::spawn`. A panic there leaves the client waiting forever for a reply.

Fix: wrap every tool handler future in `catch_unwind` and map a panic to a tool error that includes the panic message.

## The stack margin was not rechecked on an unwind build

The 8 MiB worker stack from #282 was proven on abort builds. The binary grew about 17 percent. Rerun the wheel-smoke deep trial and drug commands on yellow against the unwind release binary and record the result.

## Minor

- The join discards the panic message (`outcome.rs:617`). Put it in the error text.
- A poisoned lock after a panic disables the DDInter index cache (`src/sources/ddinter.rs:216`) and GenCC leases (`src/sources/gencc/store.rs:899`) until restart. Recover with `PoisonError::into_inner`.

## Resolved

Ticket 1235 advances rejected matches by a UTF-8 character, catches panics at the outer MCP tool dispatch, proves same-session recovery through an internal environment-guarded tool, preserves panic payload text, and routes all five reconstructible lock acquisitions through one tested poison-recovery helper. The yellow gate and unwind release-binary margin run remain the ticket's merge checks.
