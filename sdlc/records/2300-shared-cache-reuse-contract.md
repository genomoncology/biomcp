# Share admitted cache reuse assertions

Starting revision: `3819a97ab`, based on main `91118b588`.

The outcome is one shared Python helper that snapshots actual HTTP cache admission and rejects a later provider request for any admitted identity. Missing concurrent best-effort writes may refetch. Returned answers, explicit uncontended seeds, distinct request identities, database integrity, pagination and mandatory request coalescing remain required.

Before changes, `rg -n 'admitted_provider_keys|all parallel starts must reuse|both processes must reuse' tests/test_http_cache_fail_soft.py tests/test_cache_sqlite.py` finds the local helper and three independent reuse assertions. The existing controlled skipped-write failure and real `--no-cache` sensitivity are documented with exact job outputs in record2285. That evidence proves the admission-policy mismatch; it does not prove the new shared helper. No new execution has run yet.

Changes: `tests/golden/cache_reuse.py`, `tests/test_http_cache_fail_soft.py`, `tests/test_cache_sqlite.py`, `CHANGELOG.md`, and this record. The coordinator granted the sqlite writer before its change. Shared replay environment and failed-case ownership remain outside this change.

Given an admitted gene answer and a contended article write that cannot be admitted, when the CLI repeats both searches, it must return the same answers, reuse the gene and refetch the article. The same helper must reject a real `--no-cache` gene request. This outside-in control will run with the affected existing tests through a scheduled build job. The preimage predicate that all previously contacted responses must be reused must reject the allowed refetch.

The source sweep found three Python best-effort admission/reuse consumers: article/search-all, current-epoch concurrent readers, and initialized MCP/CLI sharing. All use the shared assertion. Explicit primary admission remains required for the reader case. The MCP case keeps its two distinct rows, keys, base keys, escaped query identities and integrity check after uncontended seeds.

The remaining counts protect different promises and stay unchanged: committed byte accounting in `test_cache_persistent_size.py`; sequential settled refills after cache clear in `windows_stdio_contract.rs`; shared construction in `src/sources/tests/provider_network.rs`; HTTP request identity in `src/cache/sqlite/tests.rs`; and disabled-source network absence. Semantic pagination, request planning and mandatory coalescing counts are outside best-effort HTTP admission.
