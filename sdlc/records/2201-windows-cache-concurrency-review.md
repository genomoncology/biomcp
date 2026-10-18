# Windows shared-cache concurrency review

The manual Windows run [39159496454](https://github.com/genomoncology/biomcp/actions/runs/39159496454) passed on revision `b85880f25dc489c725ab85ad82eb9f6cba66c902`, using `windows-2022`, Rust 1.93.1 and `--no-default-features`. Dispatch selected `windows_only: true`; unrelated CI jobs were skipped. No automatic workflow triggers were added.

## Observed behavior

`windows_stdio_contract` passed both tests in 15.48 seconds, including `windows_parallel_cli_and_mcp_survive_shared_cache_clear_and_keep_byte_totals`. The new case exercises the built CLI and a persistent stdio MCP server against an existing recorded MyGene response served on loopback, using a temporary shared cache.

Request-arrival barriers hold distinct cold requests from two CLI processes and the MCP server concurrently. After those requests complete, CLI and MCP warm reads require no additional provider requests. A subsequent CLI/MCP pair remains in flight when a cache-clear process is spawned; replies are then released without requiring clear to finish first. Clearing preserves the unrelated downloads sibling. Both surfaces refill and reuse the cache afterward. The persistent size marker parses completely and covers the stored content bytes after concurrent writes and after refill; conservative overcount remains allowed.

The existing pure-JSON stdio regression also passed. The separate Windows cache-lock contract passed one test, and `construction_under_a_held_epoch_lock_honors_the_deadline` passed one test.

## Review correction and local checks

The existing workflow contract still expected the previous manual-input description. Its assertion failed before the correction. Updating that assertion, requiring the stdio target in the existing command check, and extending the existing failure-injection cases to all four commands produced 30 passing tests across `test_ci_workflow_contract.py` and `test_shell_workflow_lint_contract.py`. Ruff and formatting of the Windows Rust test also passed in build job `2201/1792352829142-87313` against revision `1e2dadee0` plus the test-only patch. That base has the same Windows test and workflow as the Windows-tested revision.

The first static job then stopped because the direct workflow-lint command could not find actionlint. The remaining actionlint and diff checks passed in build job `2201/1792352847169-89399` using the repository's pinned-tool bootstrap; no behavior tests were repeated.

## Limits

The case proves the tested request overlap and successful clear/refill behavior, without asserting which process acquires the clear lock first. It uses recorded local replies rather than live providers and does not claim exhaustive Windows scheduling coverage.

One pre-existing library filter, `body_limit_epoch_lock_contention_obeys_variant_article_deadline`, selected zero tests. The test had been renamed to `body_limit_epoch_lock_contention_preserves_variant_article_time`; that filter provides no deadline evidence in this run. [The selector repair](../tickets/2219-repair-windows-deadline-test-selection.md) tracks that separate gap. The concurrency and stdio tests named above did execute and pass.
