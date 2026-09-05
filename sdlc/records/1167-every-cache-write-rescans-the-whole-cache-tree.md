---
flow: build
priority: 9
---

# Every cache write recursively rescans unrelated cache state

## Goal

Bound permission hardening after an HTTP cache write to the exact persistent
paths created or changed by that write. Preserve recursive repair at HTTP-client
construction and explicit whole-cache maintenance, without changing cache
layout, eviction, initialization scans, or provider request work.

## Result

`SizeAwareCacheManager::put` no longer recursively visits the accumulated cache
tree. Before delegation it validates and secures the managed `http` and `tmp`
directories, the known `content-v2/sha256` roots, every derived `index-v5`
ancestor, and the exact index bucket. After delegation it rereads CACache's
metadata, while retaining a shared per-cache operation lock and an exclusive
hashed key lock, to validate and secure only the exact content
algorithm/shard ancestors and blob. The key lock serializes same-key BioMCP
puts and deletes, while destructive maintenance and eviction take the cache-wide
lock exclusively. Metadata and content attribution therefore cannot change
between delegation and hardening. Missing post-write metadata fails closed
because the content path cannot otherwise be secured.

CACache's pinned SHA-1 index-bucket and integrity-derived content layouts now
live in one cache layout module shared by the manager and planner. Exact
directory creation rejects links and platform reparse points instead of walking
through them; exact files are opened without following links, checked for a
single link, and repaired privately. Recursive startup and maintenance repair
still skips unrelated links, but callers explicitly identify the concrete
managed `<cache-root>/http/content-v2` path whose directory links must be
rejected. No security behavior is inferred from a configured path's basename.

The focused cache suite passed 147 tests before the final constructor-cleanup
regression was added; that regression passed separately. Separate HTTP-client
construction coverage passed 3 security tests, and the existing managed-state
permission integration test passed. Regressions prove unrelated permissive and hard-linked
sentinels are not inspected by `put`, exact output modes remain private under a
permissive umask, every named Unix ancestor and exact output remain private,
live CACache temporary files are born `0600`, ASCII and non-ASCII/URL keys select
the derived bucket, hostile index ancestors/buckets fail before delegation, and
a hostile content destination is replaced without changing its outside target.
Deterministic concurrency regressions prove same-key puts and eviction wait for
the active operation through its post-delegation hardening window. Another
regression proves an unrelated nested directory named `content-v2` retains the
documented symlink-skip behavior. A parameterized regression proves configured
cache roots named exactly `http` and `content-v2` do not alter that boundary,
while their actual nested `http/content-v2` trees remain strict. `make lint` and
`git diff --check` passed before the lock-granularity correction. After that
correction, focused Clippy with warnings denied and `git diff --check` passed;
the full lint gate was not repeated.

An integration attempt of the earlier exclusive-lock design exposed a severe
performance regression: parallel ordinary tests queued on the single cache-root
lock while each constructor recursively repaired a 75,000-entry developer
cache. The corrected hierarchy gives constructor repair and independent key
operations shared cache-wide access, with a bounded 256-shard exclusive key
lock after the shared lock. Deterministic ordering tests prove two constructor
repairs and operations in different key shards overlap, while same-key puts
remain serialized. Migration and epoch setup acquire the exclusive cache-wide
lock only when mutation is required. Constructor age cleanup is opportunistic:
under contention it defers rather than queueing another whole-tree scan, and a
regression proves a later uncontended constructor reclaims the expired entry.
Explicit clean/clear and eviction remain cache-wide exclusive. All paths acquire
the cache-wide lock before a key lock; maintenance never acquires key locks.
The managed-state integration check also exposed and then verified the repair
for a stats-path lock upgrade: epoch establishment now finishes before stats
acquires its shared repair/snapshot gate, so no shared-to-exclusive upgrade is
attempted.

The network-dependent large-cache live canary was not run. Primary-agent
`make test` and `make spec` gates remain pending and are not claimed here.
Windows-only tests cover bounded unrelated traversal and require a protected,
current-user-only full-control ACL on each touched lock, tmp, index, and content
path. They were not compiled or executed on this Unix host: the installed Rust
Windows target cannot build the `ring` dependency because
`x86_64-w64-mingw32-gcc` is absent. Windows runtime evidence therefore remains
pending.

## Review

- Design review: accepted after multiple security corrections established the
  narrow lifecycle ownership, path-link boundaries, CACache layout contract,
  temporary-file requirement, and excluded concurrent path-swapping threat.
- Code review: reviews rejected the post-write attribution, recursive-link
  classification, Windows/ancestor coverage, the basename-derived
  managed-content context, and the global-exclusive constructor serialization.
  Those findings were remediated; independent re-review is pending.

## Boundary

This change removes the repeated post-write whole-tree multiplier and prevents
concurrent constructor repairs from serializing behind it. Startup repair and
initialization size scans remain; contended constructor age cleanup is deferred
to a later uncontended constructor or explicit maintenance. Explicit cache
maintenance, cache layout, eviction policy, cleanup planning, and provider
request work are otherwise unchanged.
Ticket 1150 continues to own the total variant-literature work budget. The
implementation does not claim race-proof filesystem behavior against concurrent
external path replacement.
