# 2133 — Stop every command walking the whole HTTP cache

Status: OPEN.

Milestone: 0.9.3

Reviews: revision a3c232c, accept

## Outcome

A command's start-up cost does not grow with the number of files in the HTTP cache. With a large cache, `biomcp get gene BRAF` and `search article` start as fast as with an empty one, and `BIOMCP_CACHE_MODE=off` touches no cache file.

## Evidence

Measured 2026-10-16 on a Linux workstation with a release build of main at db4efa43d and an existing 842 MB cache holding 102,589 files under `http/`:

- `biomcp search article -g BRAF --limit 3` took 11.4 s wall, 8.3 s of it system time, on repeated runs; with `BIOMCP_CACHE_MODE=off` it still took 10.8 s with 8.0 s system time.
- `strace -c` on `biomcp get gene BRAF`: 111,519 `openat`, 214,331 `statx` and 222,980 `getdents64` calls. The opens cover every file under `http/content-v2/sha256/`.
- The first search after replacing an older install timed out on all four article sources, then later runs succeeded.
- 2103 found the start-up secure, migrate and lock steps contend under load; 2092 limited only the orphan clean to once a day. `src/cache/private.rs` has `secure_managed_tree` and `secure_entry(path, recurse, …)`, which walk a tree.

- Starts from: `src/cache/private.rs`, `src/cache/manager.rs`, `src/cache/limits.rs`, `src/cache/migration.rs`, the 2092 daily marker.
- Decisions:
  - Find which start-up step opens every cache file and record it here.
  - Secure each file when it is written, which `secure_written_content` already does; walk the tree only when the cache root is created or migrated, recorded by a marker as 2092 does, never on every command.
  - A size or disk-floor check reads totals it keeps, or samples, rather than statting every file each run.
  - Cache mode off skips every cache start-up step.
- Changes: claims `src/cache/**` and its tests, `CHANGELOG.md`.
- Proof: a test builds a cache of several thousand files, runs one command, and asserts the number of cache files opened does not grow with the cache, failing on current main; a cache-off run opens none. On the Linux gate host, record `get gene BRAF` wall and system time with a 100,000-file cache before and after.
- Order: before release; shares `src/cache/` with nothing open.
