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

## The watch, recorded 2026-09-26 (not reopened)

The original failure is now identified, from the test's own shape:
the check sampled one 200 ms heartbeat window and compared it against
a sample taken before the fixture ran. That sampler could not
distinguish a decoy that was merely slow under load from a decoy the
fixture had wrongly killed — and the second case is a product bug,
not a flake. Nobody reproduced the original one-off, and no
reproduction is claimed. The 1252 conversion is the watch: the reap
check now reads `/proc/<pid>/stat` for alive-and-not-zombie and waits
on the stale process being gone through the scaled helper, so a
wrongly killed decoy FAILS the test deterministically — the product
bug, if it ever existed, is now distinguishable forever — and the
stress lane re-runs the test three times per invocation under forced
four-way contention on the pinned CPU set. If the product bug was
real, the lane or the direct check surfaces it; if neither ever does,
the original incident was the load case, which the signal-based wait
no longer depends on.
