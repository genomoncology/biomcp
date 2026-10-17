---
priority: 2
---

# 2172 — Let a cache failure fall through to the provider

Status: OPEN.

Milestone: 0.9.3

Depends on: 2170

## Outcome

A failure in the HTTP cache never makes a provider look unavailable: a cache setup, read or write failure logs a local cache warning and the request goes to the provider uncached, or keeps the provider's answer it already has.

- Evidence: found by the sweep after GitHub issue 292, where every run succeeded with `--no-cache`. Client construction propagates every cache step with `?` (`src/sources/mod.rs:1374-1394` and `1463-1500`: epoch lock, shared operation lock, securing the tree, start-up clean through `src/cache/manager.rs:138` and `524-542`), so any local cache problem aborts the client and the source reports unavailable. A cache read that fails to secure the tree is an error (`src/cache/manager.rs:263-290`), and an entry that fails bincode decoding propagates through http-cache. The cache write (`src/cache/manager.rs:308-331`) propagates key-lock, write and commit errors after the provider has answered, so a full disk or a lock timeout throws away a good response.
- Starts from: `src/cache/manager.rs`, the cached client construction in `src/sources/mod.rs`, and the cache deadline tests.
- Keeps: deadlines, the cache's security checks on its own tree (a tree that fails them is bypassed, never trusted), cancellation safety around publication, `--no-cache`, and every existing cache hit.
- Changes: claims `src/cache/manager.rs` `src/sources/mod.rs` `src/cache/` `CHANGELOG.md`. When cached construction fails, build the uncached client and warn once with the local cause. Treat a read failure or an undecodable entry as a miss. Return the provider's answer when the write fails, and warn.
- Proof: outside-in, through `search article` and one other entity command against the recorded-reply server. On current main, a held epoch lock past the deadline, an undecodable entry and a failed write each make the source fail or report unavailable. After the change, each returns the provider's answer and logs one local cache warning naming the cause.
- Defers: retrying Windows sharing violations on rename and remove.
