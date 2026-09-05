---
flow: build
priority: 9
---

# Every cache write recursively rescans unrelated cache state

## Goal

The permission-hardening work performed after writing one HTTP response is bounded by the paths that write created or changed. Today each successful write recursively revisits unrelated cache entries, so a multi-write request repeatedly pays for the whole accumulated tree.

Reconfirmed at commit `b2e05326`: `SizeAwareCacheManager::put` in `src/cache/manager.rs` calls `super::secure_managed_tree(&self.inner.path, true)` after each successful write. The `true` argument turns on recursion. `secure_entry` in `src/cache/private.rs:275` then reads every directory and stats every file under the cache root to confirm each one is `0700` or `0600`. One cache write walks the entire tree.

The hardening is correct and must stay. A cache entry holding provider responses must not be world-readable. Only the whole-tree sweep after every write is wrong.

This is not the only whole-tree work in cache lifecycle. Client construction currently performs a recursive permission repair in `src/sources/mod.rs`, and manager initialization snapshots entries and seeds size accounting. Those startup costs are deliberate and remain out of scope. This ticket removes the per-write multiplier; it does not promise that total command cost is independent of cache size.

## Evidence

Measured on 2026-09-05 against `~/.cache/biomcp`, which held 636 MB across 75,853 index files and 422 content blobs.

One `biomcp --json variant articles "TP53 c.356C>A" --strategy union --limit 10` under `strace -f -c` issued 927,163 `openat`, 1,247,740 `getdents64`, 828,788 `statx`, and 623,890 `fstat` calls. Almost all of the opened paths were under `http/index-v5/`. The count is about twelve full passes over the index, matching the number of cache writes the request makes.

The same command for `APC c.847C>T` was run twice back to back against the same network:

| Cache tree | Wall | User CPU | System CPU |
|---|---|---|---|
| Empty directory (19 files after the run) | 11.44 s | 0.02 s | 0.03 s |
| Real cache (75,853 index files) | 31.10 s | 4.80 s | 12.55 s |

CPU went from 0.05 s to 17.35 s for identical work. Wall clock nearly tripled.

The live gate fails because of this. `make verify` on `f68d8832` reports:

```
FAIL Provider-specific strict query provenance (line 89) [bash]: run "provider-strict-query-live-canary" timed out after 180 seconds
FAIL Provider-specific strict query provenance (line 93) [json]: run "provider-strict-query-live-canary" timed out after 180 seconds
56 passed, 2 failed, 19 skipped
```

`spec/fixtures/run-variant-article-strict-live-canary.sh` took 207 seconds against the real cache and 122 seconds against a fresh `BIOMCP_CACHE_DIR`. The spec allows 180. A developer with an accumulated cache sees a red gate. A fresh checkout sees a green one.

## Desired functionality

A cache write secures the exact paths it wrote or created. It does not inspect unrelated entries. The post-write permission work and syscall count are bounded independently of the number of existing cache entries.

The ownership of permission repair is explicit. HTTP-client construction and explicit whole-cache maintenance (`cache stats` and `cache clean`) retain full-tree repair. After client construction, an external permission change to an untouched entry is repaired by the next client construction or explicit maintenance operation, not incidentally by an unrelated write. Every persistent path touched by a write is private before that write returns.

Lifecycle ownership is split deliberately: client construction secures the cache root and performs recursive repair; each put validates and secures its managed `http` directory, persistent `tmp/` directory, exact `index-v5` key bucket and shard ancestors, and exact `content-v2` blob plus algorithm/hash shard ancestors. CACache's random atomic temporary file has been persisted or removed before `put` returns, so it must be born private rather than rediscovered afterward. Deriving persistent paths must use one centralized layout contract. If CACache does not expose the index bucket derivation, pin and test the `index-v5/<sha1[0:2]>/<sha1[2:4]>/<rest>` contract rather than rediscovering it by walking a subtree.

## Success criteria

- A deterministic regression initializes a manager, then creates an unrelated nested sentinel with permissive permissions, performs a real cache write, and proves the sentinel's bytes and mode or ACL remain identical. An operation-count or injected traversal seam also proves the write path did not inspect that unrelated entry.
- The same regression proves the exact written index bucket and content blob end at `0600`, and persistent ancestors including `http/tmp` end at `0700` on Unix, including under a permissive umask. Atomic temporary files are created private; a post-write test does not pretend the vanished random path can be inspected.
- Representative ASCII and non-ASCII or URL-shaped cache keys prove the derived SHA-1 bucket path is the exact file CACache writes. Existing duplicated `index-v5` and `content-v2` path knowledge is centralized behind that tested layout contract.
- A symlink or platform reparse point and a hard link placed at an exact derived bucket or blob path fail closed without following or changing the target. Unrelated linked entries remain untouched by put and are owned by full-tree repair.
- Existing symlink, hard-link, and permissive-umask security regressions keep passing.
- Separate regressions prove ordinary HTTP-client construction and explicit whole-cache maintenance still repair a pre-existing permissive sentinel.
- On Windows, the touched paths receive the current-user protected ACL without traversing unrelated cache entries; existing Windows security behavior remains covered.
- The large-cache live canary is recorded as supporting completion evidence. Its 180-second outcome is not the sole regression gate because provider latency varies and ticket 1150 separately owns the request's total work budget.

## Boundaries

This ticket bounds post-write permission work. It does not remove startup repair or initialization scans, change the cache layout, the eviction policy, the size accounting in `estimate_cache_bytes_fast`, the cache clean planner, or the network work a variant-literature request performs. Ticket 1150 owns the total work budget for that request. Fixing this one alone does not make the union route fast; it removes the repeated whole-tree multiplier on top of it.
