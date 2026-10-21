---
base: 8a78b20e
head: 892260d8
---

Provider-capture capacity is now an injected nonzero byte limit selected by the
production constructor. A pure planner covers exact, plus-one, and stable LRU
ties over in-memory metadata. The remaining filesystem smoke writes
kilobyte-scale captures, reopens the store under a smaller limit, performs the
real locked/fsynced eviction path, and verifies the oldest capture is gone.

Article-session expiry and count pruning now share one in-memory function with
an injected maximum. Exact, plus-one, and equal-time ordering use four entries
instead of rewriting the atomic store 1,030 times. Existing malformed-store,
locking, atomic-write, fsync, and reopen tests retain the real persistence
boundary.

The full-text transport/conversion classification matrix now uses a local
no-retry client with the production response-body limiter. One response in that
same loopback fixture passes through the production client, returns one
transient failure, succeeds on retry, and proves the retry adapter remains
wired. Existing recording-sleeper tests retain the exact delay sequence, and a
new pending-sleeper test proves cancellation.

Before the change, the three target tests took 36.369s, 14.425s, and 13.335s.
Afterward their focused runs took 0.14s for all four capacity tests and 0.65s
for the full classification/retry matrix. On exact commit `892260d8`, all 2,838
Rust tests passed in 9.465s of nextest execution, down from 36.474s before the
change (3.85x). The source change is +139 net lines, within the ticket's +140
limit, and complete lint passed.
