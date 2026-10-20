# Concurrent CLI and MCP cache qualification

The initialized MCP and CLI cache test required two cold concurrent requests to leave two stored entries. The supported cache policy permits a bounded write to skip admission under SQLite contention. One serial request after those concurrent requests admitted only its own answer, so the old assertion could reject supported behavior.

## Diagnosis

The work starts from published ticket base `9419156f0d4e0f6f0fae1c9a602b1067e081bd80`. The branch fast-forwarded to `4a970ddf8` for the coordinator's ticket progress entry before implementation. Current-main product source is `51f5e03b218c87cedc82bd9e1a67552bd5ab9df8`; the ticket branch differs from it only in the ticket record before this correction.

M5 job `2283/1792453839264-17612` used that exact current-main source and the existing local gene fixture. The probe held a SQLite `BEGIN IMMEDIATE` transaction across both cold provider responses. Both the initialized MCP call and CLI returned BRAF, identifier 673. Both logged `operation="write"` and `WouldBlock` at debug level and explicitly skipped admission. The cold database held zero entries. Neither log reported an initial manager failure. After the lock was released, the original serial CLI request for the MCP query admitted one entry. SQLite integrity passed and the unchanged two-entry assertion failed.

The two source requests carried distinct queries, `BRAF\-cli` and `BRAF\-mcp`. One uncontended admission call per query then stored two distinct keys and base keys. Subsequent identical requests reused both without another provider request. This controlled failure shows skipped admission, not a successfully acknowledged write later disappearing. The coordinator approved the correction under the bounded-admission policy already accepted for 2275.

The original Linux qualification failure remains independently retained on Yellow in `experiments/170-biomcp-2233-linux-offline-evidence/final-1792451222629-265544`. Its source scratch was removed. The retained evidence has logs, XML, source hashes and isolation evidence but no original case database. The exact missing key and original failure operation therefore remain unknown. The controlled probe does not establish the cause of that original run.

## Correction and sensitivity

`tests/test_cache_sqlite.py` keeps the initialized MCP and concurrent CLI calls and verifies their BRAF/673 identities. With both calls finished, one uncontended CLI call per distinct query admits any skipped response. The test still requires exactly two stored entries, distinct full and lookup keys, the expected source URLs and SQLite integrity. Both queries then run through the same initialized MCP process and through CLI processes with no further provider requests. A pytest-scoped response observer records only the owned fixture's server port and restores automatically. No shared helper, provider data or runtime policy changed.

The related SQLite and concurrent cache proofs were checked for the same assumption. The concurrent size-accounting case checks committed accounting, the killed-writer case seeds its answer first, and 2275 already corrected the HTTP parallel admission proof. No additional case required a change.

M5 job `2283/1792454062120-44369` applied the retained private patch to current main and reused the unchanged runtime binary. Its corrected controlled-contention case passed. A real final `--no-cache` CLI bypass failed the strict no-new-provider-request assertion. A controlled persisted lookup-key collision failed the distinct-base-key assertion. The mutations changed only private probe invocation or case databases and never entered product source.

The same job passed all 29 cases in `test_cache_sqlite.py`, `test_cache_persistent_size.py` and `test_http_cache_fail_soft.py`, Ruff for the changed Python file, tracked-text checks and the quality ratchet. No full gate or live provider request ran. M5 proves the portable contract correction; it does not supply operating-system network isolation or replace final Linux qualification.

## Retained evidence

Keep M5 `experiments/2103-biomcp-2283-concurrent-cache` for the plan, scripts, private patch, old expected-red diagnosis, controlled green, both expected-rejection traces, per-case databases, affected-case XML, job states and source parity hashes. Keep Yellow `experiments/172-biomcp-2283-concurrent-cache` for the initial queued attempt. That attempt never built or ran because memory admission waited; only its verified ticket-owned waiter was cancelled before the M5 probe.

The checked test SHA256 is `a99a90b03206e479910fff9cb6e7aa930a51dd9fae045eadbf07a91ce9a6faa2`. The checked changelog SHA256 is `867fb31c6b12940cc999bce52090631dfdc747562a418589bb92505723716682`. The unchanged focused runtime binary SHA256 is `b209cabdd54d53e5ce32df72127144d4fb0bdcd110152085fe6ad4037b1c1910`. The original Linux runtime artifact has a different hash and is not represented as the M5 artifact.
