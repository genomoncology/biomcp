# Single-CPU affinity deadlocks the pipe handshake child

Filed 2026-09-25 while gating ticket 1252. Open. Severity: P3 —
test infrastructure only; the stress lane's two-CPU pinning removes
the trigger, no production code path pins CPUs, and the failure is
deterministic rather than silent. Owner: the biomcp queue (this file
is worked when the queue reaches it). The stress lane works around it
by pinning to a two-CPU set (`taskset -c 0,1`).

## Symptom

`entities::gene::gencc::tests::subprocess_lease_child_exits_on_parent_end_of_input`
and `subprocess_lease_defers_old_generation_cleanup_until_reader_exits`
pass in 0.05 s under the normal suite (four CPUs) and under a two-CPU
set, but fail deterministically at exactly 60 s when the whole process
tree is pinned to one CPU (`taskset -c 0`), with the watchdog panic
"signaled child never reported `entered`". Both bwrap-wrapped and
bare nextest runs fail the same way, so the sandbox is not involved.

## Evidence (yellow, 2026-09-25)

- Four cells: no-pin + bwrap PASS; one-CPU + bwrap FAIL (60.0 s);
  one-CPU bare FAIL (60.0 s); two-CPU bare PASS (0.05 s).
- During the one-CPU hang: the child's test thread sits in
  `read(0, ...)` — past `signal_ready_on_raw_stdout`, which would have
  panicked on a failed write or flush; the parent's reader thread sits
  in `read(9, ...)` on the same pipe inode the child's fd 1 points at;
  the parent's test thread waits on the channel. A steal-read of the
  parent's end returned end-of-stream with no bytes.

Correction 2026-09-26: the probe evidence is partly unreliable. The
"steal-read returned end-of-stream" observation cannot be true while
the child still holds the pipe's write end open — that reading was
taken from a probe shell whose own timeout and cleanup made the
output ambiguous. The reliable facts are the four-cell behavior
above (deterministic 60 s watchdog at exactly one CPU; instant pass
at two or more) and the wchan snapshots (child's test thread in a
pipe read, parent's reader in a pipe read, parent's test thread on
the channel). Those two wchan states are consistent with the child
never having written the marker. The investigation should start
there: instrument the child's write path (does write_all return?
where do the bytes land?) under one-CPU affinity before trusting any
steal-read. Until then, treat the mechanism as unknown, not as
contradictory.

## What is needed

Reproduce standalone: spawn the handshake child under one-CPU
affinity with its stdout to a file (not a pipe) and check whether the
`entered` line lands. If it does, the capture is not the culprit and
the pipe write path is. Then bisect: `--nocapture` on the child,
`strace` the child from exec (a dedicated reproducer script), and
compare futex/wait-channel traces across the four CPU cells.

## Impact

Only the stress lane's pinning choice today; the handshake is correct
everywhere else and the lane now pins to two CPUs. Do not "fix" the
test by adding sleeps; find the mechanism.
