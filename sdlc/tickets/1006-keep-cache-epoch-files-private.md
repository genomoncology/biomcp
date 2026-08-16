---
flow: quickfix
priority: 9
---

# Keep cache epoch files private

Ordinary cached execution creates `.body-limit-cache-v1` and its lock with mode 0664 on Unix, contradicting the managed-cache privacy contract. Create and repair these exact files through a no-follow/reparse-safe opened-handle primitive. It must validate that the opened entry is a regular file with one link on Unix and Windows, repair permissions through the opened handle where the platform supports it, retain current-user-only Windows ACLs, and fail closed on symlinks, reparse points, hard links, or unsupported entries. The predictable temporary marker must use private `create_new` semantics so a stale or hostile entry is never followed or truncated.

Every migration call, including the already-current fast path, must perform O(1) validation and repair of exactly the lock and any existing marker before returning. Open and validate the lock, hold it across marker recheck, repair, cache cleanup when needed, and atomic publication. Do not recursively traverse the cache root or re-resolve a pathname for a repair when opened-handle repair is available.

Focused red-green coverage belongs in `src/cache/migration.rs` and `src/cache/private.rs`; `tests/managed_state_permissions.rs` is authorized for hosted Windows proof. Unix coverage must assert mode 0600 for fresh lock, temporary marker, and published marker; repair pre-existing 0664 marker and lock under a permissive umask; preserve concurrent idempotence; and reject symlink or hard-link marker, lock, and stale temporary entries without changing their targets. Windows coverage must preserve current-user ACL restriction and opened-handle reparse/link-count rejection for the epoch files.
