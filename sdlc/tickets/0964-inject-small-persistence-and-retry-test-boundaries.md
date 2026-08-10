---
flow: build
priority: 6
deps: ["0957"]
---
# Inject small persistence and retry test boundaries

Several routine tests reach production limits by writing tens of megabytes,
rewriting more than a thousand session files, or sleeping through production
retry delays. Keep the same assertions while testing decisions through small,
local seams and retaining one real durability/retry boundary smoke each.

## Testability contract

- Make provider-capture cache capacity an injected nonzero byte value. Extract
  a pure eviction planner over entry metadata; production remains 64 MiB.
- Extract article-session pruning over in-memory entries with injected maximum
  count/bytes. Production values and ordering remain unchanged.
- Route full-text classification retry waits through the project's existing
  sleeper/retry abstraction, or a local no-retry client where the matrix is
  testing classification rather than policy. Production backoff remains exact.

The production constructors are the only place defaults are selected. Tests
cannot mutate process-global limits or use environment variables that could
leak across parallel cases.

## Done when

- Eviction ordering and exact/plus-one boundaries use kilobyte-scale fixtures;
  one isolated persistence smoke still writes, fsyncs, reopens, and evicts.
- Session pruning covers ties, corrupt entries, exact/plus-one limits, stable
  order, and recovery in memory; one isolated atomic-write/fsync/reopen smoke
  retains the real filesystem boundary.
- Retry matrices use a recording sleeper and assert delay sequence/cancellation
  without wall-clock waits; one tiny loopback test proves the production retry
  adapter is actually wired.
- A regression test rejects the prior large-write/1,030-rewrite/production-
  sleep mechanics without weakening behavior assertions.

## Authorized test changes

Design commits may restate provider-cache construction/eviction, session
pruning/persistence, classification retry injection, and their tests. Public
cache formats, durability order, retry counts, and production limits do not
change.

The src line ceiling may rise by at most 140 lines.
