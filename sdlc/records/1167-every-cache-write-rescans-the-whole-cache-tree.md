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
metadata, while retaining an exclusive per-cache operation lock, to validate and
secure only the exact content algorithm/shard ancestors and blob. That lock also
serializes BioMCP puts with same-key puts, eviction, deletion, startup repair,
and explicit cache maintenance, so the metadata and content attribution cannot
change between delegation and hardening. Missing post-write metadata now fails
closed because the content path cannot otherwise be secured.

CACache's pinned SHA-1 index-bucket and integrity-derived content layouts now
live in one cache layout module shared by the manager and planner. Exact
directory creation rejects links and platform reparse points instead of walking
through them; exact files are opened without following links, checked for a
single link, and repaired privately. Recursive startup and maintenance repair
still skips unrelated links, but rejects directory links only within the actual
managed top-level `http/content-v2` tree so the content tree is trusted before
writes begin.

The focused cache suite passed 145 tests. Separate HTTP-client construction
coverage passed 2 security tests and the existing managed-state permission
integration test passed. Regressions prove unrelated permissive and hard-linked
sentinels are not inspected by `put`, exact output modes remain private under a
permissive umask, every named Unix ancestor and exact output remain private,
live CACache temporary files are born `0600`, ASCII and non-ASCII/URL keys select
the derived bucket, hostile index ancestors/buckets fail before delegation, and
a hostile content destination is replaced without changing its outside target.
Deterministic concurrency regressions prove same-key puts and eviction wait for
the active operation through its post-delegation hardening window. Another
regression proves an unrelated nested directory named `content-v2` retains the
documented symlink-skip behavior. `make lint` and `git diff --check` passed.

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
- Code review: an initial review rejected the post-write attribution,
  recursive-link classification, and Windows/ancestor coverage. Those findings
  were remediated; independent re-review is pending.

## Boundary

This change removes only the repeated post-write whole-tree multiplier. Startup
repair, initialization size scans, explicit cache maintenance, cache layout,
eviction policy, cleanup planning, and provider request work are unchanged.
Ticket 1150 continues to own the total variant-literature work budget. The
implementation does not claim race-proof filesystem behavior against concurrent
external path replacement.
