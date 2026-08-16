---
flow: quickfix
priority: 9
---

# Keep cache epoch files private

Ordinary cached execution creates `.body-limit-cache-v1` and its lock with mode 0664 on Unix, contradicting the managed-cache privacy contract. Create their temporary and final files privately and repair existing marker and lock permissions without weakening locking, atomic replacement, hard-link checks, or Windows managed-state behavior.

Focused red-green coverage belongs in `src/cache/migration.rs`; related private-state assertions in `src/cache/private.rs` and `tests/managed_state_permissions.rs` may be restated if needed.
