---
base: 00cef949
head: 095c402f
---

Closed by ticket 1252's conversion; recorded here so the ticket does
not dangle.

Filed 2026-09-24 when the disease-survival reap test failed once in a
full gate run and passed three focused runs. The original ticket
guessed that suite processes raced the scan; Ian's review rejected
the guess as unevidenced and asked for a rewrite on the signal basis.
Ticket 1252 rewrote it: the single 200 ms heartbeat sample is gone,
replaced by `/proc/<pid>/stat` alive-and-not-zombie reads through the
shared `proc_alive` helper, and the wait on the stale process being
gone runs through `wait_until` with a scaled watchdog. The rewritten
test ran green in every full gate since (1252's gate and the merges
after), including the stress lane's three repeated rounds pinned to a
two-CPU set. No production code was involved.
