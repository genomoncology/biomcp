# 2134 — Move the cache into one SQLite database

Status: OPEN.

Milestone: 0.9.4

## Outcome

BioMCP's HTTP cache, saved article full text, citation evidence and provider captures live in one SQLite database under the cache root. A lookup is one indexed read, size limits and eviction need no folder walk, the CLI and the MCP server share the cache safely, and every cache-age note reads as it does today.

- Changes: `src/cache/**` `src/sources/mod.rs` `src/entities/article/graph/citation_evidence.rs` `Cargo.toml` `Cargo.lock` `deny.toml` `docs/reference/configuration.md` `CHANGELOG.md` and the existing download store and focused cache tests. Narrow the download-store and test paths after the initial inventory before editing.


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
  - Start the new cache empty: every stored reply can be fetched again. After the first successful open, delete the old `http/`, `downloads/` and `citation-evidence/` trees.
  - `BIOMCP_CACHE_MODE=off` and `--no-cache` open no database.
  - Downloaded datasets in the data folder (WHO, GTR, EMA, DDInter, CVX, GenCC) stay as they are.
- Changes: claims `src/cache/**` `src/sources/mod.rs` (cache wiring only) `src/entities/article/graph/citation_evidence.rs` the full-text download store `Cargo.toml` `Cargo.lock` `deny.toml` `docs/reference/configuration.md` `CHANGELOG.md` and the tests beside them.
- Proof: the existing cache-age, day-cache, cache-mode, size-limit and disk-floor tests pass unchanged; a test runs the CLI and an MCP server against one cache at once; a test shows a lookup with 100,000 stored replies opens one database and no reply files; on the Linux gate host, `get gene BRAF` wall and system time with a 100,000-reply cache, before and after; a crash-recovery test kills a writer mid-transaction and reopens cleanly.
- Order: after 2133.
