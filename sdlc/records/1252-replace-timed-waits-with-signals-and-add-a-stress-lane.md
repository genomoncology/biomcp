---
base: cbe6233a
head: 095c402f
---

Replaced the timed test waits with signals and added a stress lane,
from the wall-clock issue.

The GenCC lease tests no longer poll files against deadlines. The
child helper writes `entered` on the raw stdout fd (bypassing libtest
capture), then blocks reading stdin; the parent's `SignaledChild`
scans for that exact line — the scan is the wait — asserts the child
is alive, and releases it by dropping the stdin handle. A dying parent
closes the pipe at the kernel, so the child exits on end-of-input and
no orphan survives; a permanent test proves that path. The settle
helper's failure now names the leaked temporaries instead of a generic
message, and its deadline is a marked watchdog. The disease-survival
reap check reads `/proc/<pid>/stat` for alive-and-not-zombie instead
of sampling a heartbeat, and waits on the stale process being gone;
ticket 1248 was rewritten on this basis and its unevidenced race
guess withdrawn. Shared helpers carry the scale factor
(`BIOMCP_TEST_TIMEOUT_SCALE`), and a new ratchet in `make lint` pins
every existing `sleep`-family call as per-file ceilings that only
ratchet down, with a `watchdog:` marker for deliberate ones and a
mechanical rule for heartbeat-shaped helpers. `make stress` runs the
known-flaky set on a pinned two-CPU set with forced four-way
parallelism, repeated by default three times, its watchdogs scaled.

One-CPU pinning was tried first and deadlocks the pipe handshake
child deterministically (four-cell evidence: multi-CPU and two-CPU
pass in 0.05 s, one-CPU fails at exactly the 60 s watchdog with the
child demonstrably past its write and the parent's reader blocked on
the same pipe). The mechanism is unidentified and filed as
`sdlc/issues/2026-09-25-single-cpu-affinity-deadlocks-the-handshake-child.md`;
the lane pins to two CPUs and a contract test forbids ever pinning to
exactly one.

Evidence: design REJECT once (four P1s: the settle diagnostic could
not land under the size gate, the heartbeat lint rule had no matcher,
the scale factor's reach was overstated, and the stress acceptance
had no red side) — all folded, re-reviewed ACCEPT; code review REJECT
once (the ratchet failed on its own test file's literals, plus two
stale entries) — fixed and re-verified; the yellow gate failed twice
on the stress leg (the lane's own contract-test lint errors, then the
one-CPU deadlock) before landing green at e01ebf7a with lint, test,
spec, and stress all OK. The red side is the ratchet failing on the
parent commit's exact files plus the never-signaling-child unit test;
a live flake reproduction against the parent tree was attempted and
did not complete, and none is claimed.

Residuals: the one-CPU affinity deadlock (open issue, probe recipe
included); the ~12 remaining Rust and 12 Python pinned sleeps ratchet
down over time; `BIOMCP_TEST_TIMEOUT_SCALE` covers only
helper-built watchdogs by design.
