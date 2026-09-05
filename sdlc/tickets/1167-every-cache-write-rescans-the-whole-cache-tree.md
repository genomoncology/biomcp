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

Lifecycle ownership is split deliberately. Client construction secures the cache root, performs recursive repair, and rejects symlink or platform reparse-point directories anywhere under managed `content-v2` rather than treating them as repairable or skipping them. That establishes the trusted content tree before writes begin. Each put validates and secures its managed `http` directory, persistent `tmp/` directory, exact `index-v5` key bucket and every index shard ancestor before delegation. For content it can prevalidate only the known `content-v2` and `sha256` roots: the exact SRI hashes `http-cache`'s private serialized `Store` and is not available through its API until after delegation. The put therefore validates and secures the exact returned content hash shards and blob afterward. Normal derived ancestors finish at `0700` or the current-user-only Windows equivalent. External replacement of a content shard after manager construction is part of the explicitly excluded concurrent path-swapping threat, not something this wrapper can make race-proof. CACache's random atomic temporary file has been persisted or removed before `put` returns, so it must be born private rather than rediscovered afterward. Deriving persistent index paths must use one centralized layout contract. If CACache does not expose the index bucket derivation, pin and test the `index-v5/<sha1[0:2]>/<sha1[2:4]>/<rest>` contract rather than rediscovering it by walking a subtree. BioMCP must not duplicate `http-cache`'s private `Store` serialization merely to predict the content SRI before delegation.

## Success criteria

- A deterministic regression initializes a manager, then creates an unrelated nested sentinel with permissive permissions, performs a real cache write, and proves the sentinel's bytes and mode or ACL remain identical. An operation-count or injected traversal seam also proves the write path did not inspect that unrelated entry.
- The same regression proves the exact written index bucket and content blob end at `0600`, and persistent ancestors including `http/tmp` end at `0700` on Unix, including under a permissive umask. A deterministic test opens a real `cacache::WriteOpts` writer in an otherwise empty secured `http/tmp`, inspects its sole live temporary entry before commit, and proves atomic temporary files are born private; a post-write test does not pretend the vanished random path can be inspected.
- Representative ASCII and non-ASCII or URL-shaped cache keys prove the derived SHA-1 bucket path is the exact file CACache writes. Existing duplicated `index-v5` and `content-v2` path knowledge is centralized behind that tested layout contract.
- A symlink or platform reparse point, or a multiply-linked file, placed at the exact derived index bucket is rejected before CACache writes, without following or changing the outside target. The same pre-write rejection applies to every derived index ancestor. Client construction and explicit maintenance reject a symlink or reparse-point directory anywhere under managed `content-v2`; per-put checks the known content roots before delegation and the exact returned hash shards afterward.
- At the content destination, either a supported derivation API permits pre-write rejection or CACache's atomic replacement semantics may replace a hostile entry. In both cases the outside target remains unchanged and the resulting exact blob is a regular, single-link, privately secured file. This protects against pre-existing hostile entries; it does not claim a new race-proof filesystem API against concurrent path swapping.
- Unrelated ordinary permissive files and directories remain untouched by put and are repaired by client construction or explicit maintenance. Unrelated symlinks retain the existing skip behavior except that symlinked directories inside managed `content-v2` are rejected to establish the trusted content tree; unrelated hard links retain the existing maintenance failure behavior.
- Existing symlink, hard-link, and permissive-umask security regressions keep passing.
- Separate regressions prove ordinary HTTP-client construction and explicit whole-cache maintenance still repair a pre-existing permissive sentinel.
- On Windows, the touched paths receive the current-user protected ACL without traversing unrelated cache entries; existing Windows security behavior remains covered.
- The large-cache live canary is recorded as supporting completion evidence. Its 180-second outcome is not the sole regression gate because provider latency varies and ticket 1150 separately owns the request's total work budget.

## Boundaries

This ticket bounds post-write permission work. It does not remove startup repair or initialization scans, change the cache layout, the eviction policy, the size accounting in `estimate_cache_bytes_fast`, the cache clean planner, or the network work a variant-literature request performs. Ticket 1150 owns the total work budget for that request. Fixing this one alone does not make the union route fast; it removes the repeated whole-tree multiplier on top of it.

Implementation-gate evidence corrected one initialization detail. A single
global exclusive lock serialized parallel constructor repair and cleanup scans
over a 75,000-entry cache, making the ordinary Rust gate stall. Startup repair
and size estimation remain, but constructor age cleanup is opportunistic under
contention: it runs under an exclusive cache-wide lock when immediately
available and otherwise defers to a later uncontended constructor or explicit
maintenance. This is the only correction to the initialization boundary.
