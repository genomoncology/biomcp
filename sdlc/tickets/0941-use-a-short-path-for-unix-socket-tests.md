---
flow: build
priority: 6
deps: ["0949"]
---
# Use a short path for Unix socket tests

Issue owned by this ticket:
`sdlc/issues/unix-socket-cache-clear-test-assumes-short-temp-path.md`

`cache::clear::tests::clear_rejects_special_file_before_mutation` builds its
Unix socket below the ambient `TMPDIR`. A long CI or worktree path can exceed
`sockaddr_un.sun_path` and panic before the cache-clear behavior is tested.

## Done when

The Unix-socket fixture chooses a short test-owned root, verifies the complete
socket path fits the platform limit before bind, and cleans up only that root.
It does not change global `TMPDIR`, use a shared predictable path, or weaken the
assertion that cache clear rejects a socket before mutation.

A process test runs the case with an intentionally long ambient `TMPDIR` and
passes. Parallel invocations use unique paths and do not collide. Non-Unix
platforms retain their current explicit test behavior.

## Authorized test changes

Design and code commits may change the affected cache-clear fixture and shared temporary
helper only where needed for bounded Unix socket paths. Existing special-file,
symlink, regular-file, cleanup, and parallel-isolation assertions remain.

Delete the named issue when this ticket lands.

The src line ceiling may rise by at most 30 lines.
