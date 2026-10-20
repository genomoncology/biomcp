# Share admitted cache reuse assertions

Starting revision: `3819a97ab`, based on main `91118b588`.

The outcome is one shared Python helper that snapshots actual HTTP cache admission and rejects a later provider request for any admitted identity. Missing concurrent best-effort writes may refetch. Returned answers, explicit uncontended seeds, distinct request identities, database integrity, pagination and mandatory request coalescing remain required.

Before changes, `rg -n 'admitted_provider_keys|all parallel starts must reuse|both processes must reuse' tests/test_http_cache_fail_soft.py tests/test_cache_sqlite.py` finds the local helper and three independent reuse assertions. The existing controlled skipped-write failure and real `--no-cache` sensitivity are documented with exact job outputs in record2285. That evidence proves the admission-policy mismatch; it does not prove the new shared helper. No new execution has run yet.

Changes: `tests/golden/cache_reuse.py`, `tests/test_http_cache_fail_soft.py`, `tests/test_cache_sqlite.py`, `CHANGELOG.md`, and this record. The coordinator granted the sqlite writer before its change. Shared replay environment and failed-case ownership remain outside this change.

Given an admitted gene answer and a contended article write that cannot be admitted, when the CLI repeats both searches, it must return the same answers, reuse the gene and refetch the article. The same helper must reject a real `--no-cache` gene request. This outside-in control will run with the affected existing tests through a scheduled build job. The preimage predicate that all previously contacted responses must be reused must reject the allowed refetch.

The source sweep found three Python best-effort admission/reuse consumers: article/search-all, current-epoch concurrent readers, and initialized MCP/CLI sharing. All use the shared assertion. Explicit primary admission remains required for the reader case. The MCP case keeps its two distinct rows, keys, base keys, escaped query identities and integrity check after uncontended seeds.

The remaining counts protect different promises and stay unchanged: committed byte accounting in `test_cache_persistent_size.py`; sequential settled refills after cache clear in `windows_stdio_contract.rs`; shared construction in `src/sources/tests/provider_network.rs`; HTTP request identity in `src/cache/sqlite/tests.rs`; and disabled-source network absence. Semantic pagination, request planning and mandatory coalescing counts are outside best-effort HTTP admission.

## Focused execution

M5 job 2300/1792525464752-62780 passed all 32 affected cases without skips and passed Ruff on source ed92bd180. The cases include all three shared-helper consumers, a forced SQLite contention case that permits an unadmitted article refetch while requiring admitted gene reuse, and a real `--no-cache` rejection control. The runner then failed because it tried to execute the tracked-text Python file directly. Overall that first job remains failed. Mechanical-only job 2300/1792526111660-72505 corrected the invocation to `python3 tools/check-tracked-text` and passed tracked text, quality ratchet and whitespace checks; it did not repeat the Python cases.

The exact source overlay SHA256 is 4bcc6fe98035a5d6f42a64d501088174259b7e117fbf65cc8d494db12745f9fc. The immutable CLI used for focused tests has SHA256 338e94737bb9bdd5f4d72bd0d95e7adec50b34d3271feae6b920c71dab83b2ba. Both complete jobs and their commands remain under M5 experiment 2102-biomcp-2258-final-live-replay/cache-admission/2300/jobs. These focused proofs do not qualify a later dependency graph. The required routine check on a clean committed branch merged with repaired main remains pending before fresh review.

The exact clean committed source 45d3f2e8e8921805d16cee674d4d1262761601f1 passed M5 routine 2300/1792528650352-40615, including lint, tests and specifications. The runner imported the retained bundle from published base 7503f6fd, verified tree 1aa5d632 and ticket identity, and found no tracked changes after the gate. An earlier direct private-SHA dispatch was refused before execution; the corrected published-base transport ran the qualifying command. The source was then pushed once. Complete job records remain under cache-admission/2300/jobs/1792528650352-40615. Root subsequently merged reviewed main 267f583, which adds only authorized changelog and SDLC text; the helper and all test bytes remain unchanged. Darwin provided no network isolation, so this is not Linux offline evidence.
