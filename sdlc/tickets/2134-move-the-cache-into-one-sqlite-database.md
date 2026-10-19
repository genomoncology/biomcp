# 2134 — Move the cache into one SQLite database

Status: OPEN.

Milestone: 0.9.4

## Outcome

BioMCP's HTTP cache, saved article full text, citation evidence and provider captures live in one SQLite database under the cache root. Lookups use indexed rows, size limits and eviction need no folder walk, the CLI and the MCP server share the cache safely, and every cache-age note reads as it does today.

- Changes: `src/cache/**` `src/test_support.rs` `src/sources/mod.rs` `src/utils/download.rs` `src/entities/article/fulltext.rs` `src/cli/cache.rs` `src/cli/health/local.rs` `src/cli/health/mod.rs` `src/cli/health/tests/local.rs` `src/cli/gene/cspec.rs` `src/cli/gene/dispatch.rs` `src/mcp/shell.rs` `src/entities/article/graph/citation_evidence.rs` `src/entities/article/graph/tests/cache.rs` `src/cli/worker.rs` `src/entities/gene/cspec.rs` `src/entities/article/supplement_search/cache.rs` `src/sources/fda_orphan.rs` `src/sources/semantic_scholar.rs` `src/sources/tests/provider_network.rs` `src/sources/gwas/tests/day_cache.rs` `src/sources/openfda/tests/day_cache.rs` `Cargo.toml` `Cargo.lock` `deny.toml` `docs/reference/configuration.md` `CHANGELOG.md` `tests/cache_clear.rs` `tests/windows_stdio_contract.rs` `tests/test_cache_*.py` `tests/test_http_cache_fail_soft.py` `tests/test_no_cache_managed_state.py` `tests/test_gwas_day_cache_contract.py` `tests/article_asset_terminal.rs` `tests/article_asset_terminal/supplement_cache.rs` `src/cli/outcome.rs` `src/cli/tests/facade/cache.rs` `tests/test_cache_sqlite.py` `tests/test_small_rust_test_boundaries.py` `tests/managed_state_permissions.rs` `tests/fulltext_fixture_output_location.rs` `src/entities/article/search/tests/deadline.rs` `spec/fixtures/run-article-search-deadline-search.sh` `spec/entity/article.md` `spec/surface/cli.md` `docs/policies.md` `docs/troubleshooting.md` `docs/reference/data-sources.md` `docs/reference/faq.md` `docs/user-guide/cli-reference.md` `tests/test_documentation_consistency_audit_contract.py` `tools/check-quality-ratchet.py` `tests/test_quality_ratchet_contract.py` and focused new SQLite behavior tests. The new SQLite modules live under `src/cache/sqlite.rs` and `src/cache/sqlite/`. Narrow any additional path before editing.


## Development brief and migration ruling

Restated outcome: use one indexed SQLite cache with shared-process safety, preserved freshness and bounded storage, while retaining readable saved full-text artifacts and explicit exports.

Reproduction on main `08b74a340658f2c30e158fd00459330baebacd74`: job `2134/1792409854384-61696` ran the retained inline driver, using a frozen qualified CLI with SHA256 `886e80635de11125329d039da533c4e1ca7ee478936b4c0eb330bcf05d138b4c` and a synthetic loopback provider. The primary command was `biomcp --json search gene BRAF<N> --limit 1`, repeated for thirty distinct queries, then `biomcp --json cache clean`. With a 50,000-byte body budget the snapshots were six index files / 4,914 bytes, twelve / 10,756 bytes, thirty / 28,342 bytes, while retained bodies stayed at two / 34,996 bytes. Clean exited 0 with no errors and left thirty index files. The complete driver and invocation remain in the job; its owned scratch and server were removed. This proves retained index growth, not a new ordinary warm-lookup folder walk.

PM ruling, 2026-10-19: SQLite owns reusable full text, but fulltext still returns a real readable `full_text_path`. Saved downloads and explicit exports remain ordinary files and survive both cache clear and cache clean. Keep the database isolated from old-version writes. Never automatically delete legacy stores or captures. Cache stats name their sizes; the explicit `biomcp cache clean-legacy --yes` command removes only disposable legacy HTTP, evidence and capture stores on request, preserving saved downloads, the new database/blobs and sessions. Match the existing clear command confirmation convention rather than introducing a different interactive flow. The changelog explains that old versions continue writing their own stores until upgraded. Automatic legacy removal is deferred beyond this release. `cache path` reports the database for the new backend. This ruling supersedes the automatic-deletion instruction in the older architecture below.

Preserve behavioral assertions; replace storage-specific cacache file/marker assertions with the new backend contracts. The no-reply-file proof applies to small inline replies; large blobs remain permitted. Article sessions and asset manifests stay outside the migration. Request-context handling for Vary/body keys and transactional blob ownership must be covered by the implementation and fresh review, rather than approximated from the old string key.

Outside-in red cases: legacy-store coexistence and repeat fixture fetch through CLI with a real database; size eviction and concurrent initialized MCP/CLI reuse; indexed hit among 100,000 small replies; killed uncommitted writer recovery; preserved fulltext path/views/export and saved downloads across clear/clean. Use focused behavioral cases and justified storage assertions, not a second general cache framework. Run occasional Linux proof only for the ticket's named lookup timing/syscall comparison, after the ordinary implementation is ready.

PM ruling, 2026-10-19, explicit capture exception: preserve the existing `gene cspec --version` capture workflow under cache off and `--no-cache`. Explicit capture creation/read operations use only their artifact table; optional HTTP reuse stays disabled. Document this boundary and test bypass plus capture leaves zero HTTP cache entries. The blanket no-database sentence below is updated to match.

Reviewed retention detail: ordinary cache clear preserves provider capture rows, matching the existing published policy and the PM ruling that explicit captures are saved artifacts. Preserve their existing expiry/capacity maintenance behavior and the other retained artifact promises; do not clear every table indiscriminately. Backend-specific deadline, permissions and layout fixtures may change their storage mechanism while keeping their public refusal, deadline and trust assertions. Historical design documents stay historical.

Reviewed new-store trust boundary: create SQLite files with private permissions. An existing insecure database is refused with the established fail-soft provider behavior and a clear diagnostic; do not open/close an ordinary repair descriptor that can release another local connection's POSIX locks. Keep managed-root repair and symlink/hardlink rejection, plus the Windows ACL boundary. The retired arbitrary legacy-file walk is not a new-format repair contract. Verify the new store with the real permissions and escape tests.

## Evidence

- The HTTP cache is http-cache 0.20 over cacache 13. Its key is `method:uri`, the full unnormalized URL. cacache only appends index buckets, so one long-used machine held 102,376 index files (709 MB) for 212 stored bodies (120 MB) on 2026-10-16.
- Lookups, size checks and the start-up secure walk scan those files; 2133 removes the per-command walk on the current store, but the index still grows without bound.
- Other cache areas are separate file stores: article full text in `downloads/<md5>.txt`, citation evidence one file per edge (`citation-evidence/v1/`, 30-day TTL), provider captures (7 days), plus key-lock and maintenance-lock files.
- Architecture proposal reviewed 2026-10-16: one SQLite file, WAL mode, request-keyed replies with freshness from the stored HTTP policy, least-recently-used eviction from an index, a kept byte total, private permissions set once at creation.

- Starts from: `src/cache/**`, `src/sources/mod.rs` (cached client construction, around line 1415), the full-text download store, `src/entities/article/graph/citation_evidence.rs`, `src/cache/provider_capture.rs`.
- Decisions:
  - One database, `<cache_root>/biomcp.db`, opened with `journal_mode=WAL`, `synchronous=NORMAL`, a busy timeout, and 0600 files in a 0700 directory set at creation. Crate: `rusqlite` with `bundled`, called through `spawn_blocking`.
  - A `SqliteCacheManager` implements http-cache's `CacheManager`. The key is a versioned hash of the method, the normalized URL (sorted query parameters, API-key parameters dropped, lowercased scheme and host), the request headers the reply's Vary names, and a body hash. Stored policies keep ETag and Last-Modified revalidation, `infinite` mode and 2091's day cache.
  - Bodies over about 1 MB go to `blobs/<sha256>` files, written before their row commits. A trigger keeps the byte total; over the size limit or under the disk floor, delete by `last_access` through its index. `last_access` updates at most hourly per row.
  - Full text, citation evidence and captures become tables in the same file, with their current lifetimes. `cache stats`, `clear` and `clean` work against the database. The key-lock and maintenance-lock files go.
  - Start the new cache empty and keep it isolated from old-version writes. Keep saved downloads as user-readable artifacts across clear and clean. Never automatically delete legacy stores or captures; report their sizes in stats and remove only disposable legacy cache stores through the explicit documented cleanup command when requested.
  - `BIOMCP_CACHE_MODE=off` and `--no-cache` disable optional HTTP reuse and open no database for ordinary requests. An explicitly requested capture is a saved artifact: its creation and later reads still access the capture table under bypass, without creating HTTP cache entries or running optional HTTP maintenance.
  - Downloaded datasets in the data folder (WHO, GTR, EMA, DDInter, CVX, GenCC) stay as they are.
- Changes: claims `src/cache/**` `src/sources/mod.rs` (cache wiring only) `src/entities/article/graph/citation_evidence.rs` the full-text download store `Cargo.toml` `Cargo.lock` `deny.toml` `docs/reference/configuration.md` `CHANGELOG.md` and the tests beside them.
- Proof: the existing cache-age, day-cache, cache-mode, size-limit and disk-floor tests pass unchanged; a test runs the CLI and an MCP server against one cache at once; a test shows a lookup with 100,000 stored replies opens one database and no reply files; on the Linux gate host, `get gene BRAF` wall and system time with a 100,000-reply cache, before and after; a crash-recovery test kills a writer mid-transaction and reopens cleanly.
- Order: after 2133.

## Progress

- 2026-10-19 started

The migration updates the existing quality audit and its malformed-fixture contract only to replace the retired epoch-guard assertion with the isolated SQLite database contract. Preserve body-limiter ordering and all unrelated archive/custom-limit assertions. This is a backend migration adjustment, with no new audit machinery.

## Reviewed implementation and proof

Variant-aware HTTP lookup uses two indexed SQL reads: a metadata probe followed by one consistent row containing body, headers and policy, with Vary rechecked. This is the reviewed implementation of indexed lookup; it makes no literal single-statement promise. Reusable full text stays canonical in SQLite, while saved paths and exports remain ordinary readable artifacts.

The developer grouped job `2134/1792413316208-65234` passed its 143 Rust and 62 Python behavior checks, then failed its clippy phase. Later mechanical fixes and lint passed in `2134/1792413996468-82968`, together with six SQLite CLI cases and two malformed audit fixtures. Saved-path/export/views plus nine CSpec cases passed in `2134/1792413504646-88638`. Those private-patch runs retain archive hashes but no per-file manifest or archive; their correspondence to the pre-review source is procedural, not independently retained byte identity.

Fresh review found and fixed three concrete issues in e53502608eae9c67ecbbc83068e9247ffe9e5c02: clear/clean retained freed inline pages, linked blob directories could redirect deletion outside the cache, and concurrent replacement could mix old headers with a new body. Red jobs were `2134/1792414424029-47294` and `2134/1792414644356-85486`. The fix configures vacuum before WAL, steps reclamation to completion, calls it after clear, validates shared blob directory boundaries and returns headers/policy with the body row. Existing cancellation, trust, capture and artifact promises remain.

Final published-revision job `2134/1792414873427-14533` passed 15 focused Rust tests, ten SQLite CLI tests, formatting, clippy and Python lint. It records revision, source tree 76d8d20dbcb882bc43e206a4fac07724698bc688 and nine file hashes that the reviewer independently matched. Review scratch was removed.

The bounded Linux 100,000-entry comparison `2134/1792413723057-64471` retained unchanged command output and reduced traced cache opens from 21 to 12, reply opens from two to zero and index opens from five to zero. Its complete driver remains in `2134/1792414108498-72043`. Warm wall samples were comparable, with no strong speed claim. These observations are procedurally tied to the pre-review snapshot and do not qualify the later reviewed revision. No extra timing run is warranted. Whole-candidate qualification remains required.

Current main was merged after review; the only conflict was the changelog. Resolution preserves current main and the unchanged migration bullet. No cache implementation changed during that merge.
