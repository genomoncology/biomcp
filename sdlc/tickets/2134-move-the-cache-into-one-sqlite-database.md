# 2134 — Move the cache into one SQLite database

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 9706ad7, accept

Reviews: revision e0b4f1b, accept

Reviews: revision caaa01b, accept

## Outcome

BioMCP's HTTP cache, saved article full text, citation evidence and provider captures live in one SQLite database under the cache root. Lookups use indexed rows, size limits and eviction need no folder walk, the CLI and the MCP server share the cache safely, and every cache-age note reads as it does today.

- Changes: `src/cache/**` `src/test_support.rs` `src/sources/mod.rs` `src/utils/download.rs` `src/entities/article/fulltext.rs` `src/cli/cache.rs` `src/cli/health/local.rs` `src/cli/health/mod.rs` `src/cli/health/tests/local.rs` `src/cli/gene/cspec.rs` `src/cli/gene/dispatch.rs` `src/mcp/shell.rs` `src/entities/article/graph/citation_evidence.rs` `src/entities/article/graph/tests/cache.rs` `src/cli/worker.rs` `src/entities/gene/cspec.rs` `src/entities/article/supplement_search/cache.rs` `src/sources/fda_orphan.rs` `src/sources/semantic_scholar.rs` `src/sources/tests/provider_network.rs` `src/sources/gwas/tests/day_cache.rs` `src/sources/openfda/tests/day_cache.rs` `Cargo.toml` `Cargo.lock` `deny.toml` `docs/reference/configuration.md` `CHANGELOG.md` `tests/cache_clear.rs` `tests/windows_stdio_contract.rs` `tests/test_cache_*.py` `tests/test_http_cache_fail_soft.py` `tests/test_no_cache_managed_state.py` `tests/test_gwas_day_cache_contract.py` `tests/article_asset_terminal.rs` `tests/article_asset_terminal/supplement_cache.rs` `src/cli/outcome.rs` `src/cli/tests/facade/cache.rs` `tests/test_cache_sqlite.py` `tests/test_small_rust_test_boundaries.py` `tests/managed_state_permissions.rs` `tests/fulltext_fixture_output_location.rs` `src/entities/article/search/tests/deadline.rs` `spec/fixtures/run-article-search-deadline-search.sh` `spec/entity/article.md` `spec/surface/cli.md` `docs/policies.md` `docs/troubleshooting.md` `docs/reference/data-sources.md` `docs/reference/faq.md` `docs/user-guide/cli-reference.md` `tests/test_documentation_consistency_audit_contract.py` `tools/check-quality-ratchet.py` `tests/test_quality_ratchet_contract.py` `.github/workflows/ci.yml` `tests/test_ci_workflow_contract.py` `src/utils/sync.rs` and focused new SQLite behavior tests. The new SQLite modules live under `src/cache/sqlite.rs` and `src/cache/sqlite/`. Narrow any additional path before editing.


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

## Landing gate repair brief

The whole-candidate job `2134/1792415243869-81937` stalled in `entities::article::search::tests::deadline::fast_pubtator_failure_does_not_hide_the_deadline_error`. Its owned nextest child remained blocked with zero CPU while all other children finished. The coordinator stopped only that verified child, producing a failed gate after 538 seconds. This is an explicitly interrupted stalled test, not an assertion failure or a passing suite. Gate output remains under gate-runs/biomcp/2134/m5/f9117baf5348ae49bdaba2c72cd12481eea60660. Candidate source did not reach main; actual main 096c11fc416ddc18ee3b05814a6d0b9fede63648 has a passing full check.

One fresh developer owns the bounded cause and repair within the existing declared paths, including the changed article deadline test. Preserve its provider failure, deadline and cache promises. Compare the test and real cache behavior against main, reproduce with a bounded focused job, fix the actual cause and run affected cases. Do not hide a hang with sleeps, relaxed assertions or a larger product deadline. Do not rerun the whole suite while diagnosing. Request any new path before editing it. Other accepted work continues from green main while this lane repairs the candidate.

The same failed gate also reported a Windows selector naming the retired body_limit_epoch_lock_contention_preserves_variant_article_time test. Update only the two declared selector paths to the surviving SQLite deadline/contention proof, construction_under_a_held_sqlite_writer_honors_the_deadline and sqlite_busy_deadline_finishes_without_late_publication. Preserve the CI trigger policy and other selections. No Windows dispatch or release redesign is authorized by this mechanical migration fix.

## Candidate repair proof

The fresh repair reproduced the stalled shutdown in bounded job `2134/1792416069699-3129`, which exited 124. Its retained deadline.sample shows SQLite open and close workers waiting on opposite mutexes while Tokio runtime shutdown waits for them. The earlier deadline-marker hypothesis was superseded by this captured stack and the upstream SQLite defect. SQLite documents the deadlock fix in [3.51.2](https://sqlite.org/releaselog/3_51_2.html).

Repair e9adfb81eb389b2e0ef0fc132f6e7b134b1935c3 changes only Cargo.toml, Cargo.lock and the two authorized selector paths. Rusqlite 0.40.2 bundles SQLite 3.53.2. Application code, Rust assertions and deadline values are unchanged. The Windows test-name extractor now recognizes intervening test attributes while preserving real-selector validation.

Job `2134/1792416295225-34366` passed 31 affected tests and 30 repetitions of the previously hanging test, plus lint; its terminal workflow-parser failure was fixed afterward. `2134/1792416498097-98032` passed all 27 workflow contracts and the final quality ratchet. Published verification `2134/1792416573100-25877` matched the tested source hashes, including the unchanged deadline file. Exact repair patch SHA256 3879e25ce349a84c8f277a0bf306ee1f47284ae1b44f604dcaeb4f378ee6bb24. Scratch was removed, diagnostic stack retained. Fresh review and whole-candidate qualification remain required.

Declared compiler compatibility is now verified: `2134/1792416972901-69562` passed `cargo +1.89.0 check --locked --no-default-features --lib` on reviewed revision 24d17ba84b871df60d43665c27be8675071e4361, including rusqlite 0.40.2 and the application library. The log records rustc 1.89.0 (29483883e 2025-08-04), source and matching manifests. The isolated temporary toolchain and target were removed; shared toolchains were unchanged. This supersedes the earlier unverified-compiler limitation.

Published main 478bcb9c6127d9c749a0f60b8ad3d4732b7e9c10 was merged cleanly after its full check passed. Only its accepted trial specification and ticket records are newly inherited; the reviewed cache implementation and repair are unchanged.

## Shared-lock gate finding

The repaired candidate job `2134/1792417446224-21102` completed without the shutdown hang. It passed 1337 Python cases; Rust stopped after 4080 passing cases on `utils::sync::tests::every_reconstructible_lock_uses_the_shared_helper`, which asserted zero versus three at src/utils/sync.rs:35. This is the next bounded migration finding, not a main failure. Preserve the shared-lock recovery convention; determine whether a real caller violates it or a retired caller remains in the audit before choosing the minimal correction. The exact additional path is declared above. Do not remove the convention or change the production helper without a demonstrated defect. Use focused proof, then fresh review; no broad investigation or repeated full suite during diagnosis. Whole-candidate output is gate-runs/biomcp/2134/m5/e597b72ca0798ab371760ee260a8ab95ac53e153. Main remains qualified at 478bcb9c6127d9c749a0f60b8ad3d4732b7e9c10 while other accepted changes land.

The shared-lock finding is a retired caller inventory: cache/clear.rs no longer contains its old three recoverable acquisitions, while cache/sqlite.rs has one surviving OPEN acquisition that already uses recover_poison. Repair 33cda4ce5447ca275b8130af44dfe55a246a5bb2 updates only that audit's include and expected count. The production helper and all callers remain unchanged. Focused `2134/1792417831255-77504` passed both sync tests, formatting, all-target clippy and boundary checks. Published verification `2134/1792417970007-1723` matched source hashes and exact patch SHA256 daafd7c971c4de0f1046621105bbc5b695315957873149c89a927d84fdfed9db. Scratch was removed. The earlier brief's right-hand count was corrected to three from the raw evidence.
