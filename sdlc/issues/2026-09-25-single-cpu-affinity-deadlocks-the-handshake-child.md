# Single-CPU affinity deadlocks the pipe handshake child

Filed 2026-09-25 while gating ticket 1252. Open. The stress lane works
around it by pinning to a two-CPU set (`taskset -c 0,1`).

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

That combination — the child demonstrably past its pipe write, the
parent's reader blocked on that pipe, no bytes arriving — is not
explainable by the kernel pipe semantics we assume. Something between
the child's `std::io::stdout().write_all` and the pipe write end is
affinity-sensitive in a way the static reading misses (libtest output
capture interacting with the raw-fd write is the untested candidate).

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
