---
flow: build
priority: 1
deps: [1190]
---

# 1193: lifecycle runner spawns survive inherited ignored signals

## Goal

Every runner-termination lifecycle test is deterministic in any launcher
environment: the spawned `run-specs.sh` runner always honors the SIGHUP,
SIGINT, and SIGTERM traps the script installs, regardless of which fatal
signals the launching process chain happened to ignore.

## Current Facts — the mechanism, proven 2026-09-14 on the dev host

The documented "CPU-contention" class is not contention. Controlled
experiments with a second-by-second process monitor, /proc signal-mask
capture, and a minimal bash reproducer established:

1. POSIX forbids a non-interactive shell from trapping a signal that was
   ignored when the shell started; bash honors this silently. A runner
   spawned with a fatal signal already SIG_IGN has that `trap` line become a
   no-op, and `os.kill(runner.pid, sig)` can never terminate it.
2. Ignored dispositions inherit through the whole launch chain: `nohup`
   ignores SIGHUP; pytest-xdist worker chains and sandbox wrappers can leak
   others. The stuck runner captured live showed `SigIgn=0x5` (SIGHUP and
   SIGQUIT ignored) and cycled fresh `sleep 1` children for the full
   sixty-second wait — the trap never fired because the signal could never
   be caught.
3. The failing parametrizations map exactly to the leaked set: pytest ids
   are signal values, so `[1]` is SIGHUP (the nohup-launched gate scripts'
   leak) and the gate host's historical `[2]` failures are SIGINT leaking
   through its chain. `[15]` (SIGTERM) never leaked on either host.
4. Load was never the variable: under twelve CPU spinners and four pytest
   workers, the full trio passes 37/37 in forty seconds when launched in the
   foreground (`load average 26`); with `nohup`, the same suite fails every
   `[1]` parametrization while solo and idle runs pass. The apparent load
   correlation came from gate lanes always running under nohup.
5. A minimal bash reproducer confirms the fix direction: with the parent
   ignoring SIGINT, a trapped child hangs despite the trap; with the
   disposition reset at spawn, it exits in half a second under the same
   load.

## Scope

Add a spawn-time disposition reset in the three lifecycle test files: a
preexec helper that sets SIGHUP, SIGINT, SIGQUIT, and SIGTERM to SIG_DFL in
the runner child, applied at the signaled runner spawn sites (article x2,
ctgov x1). The disease family signals with SIGKILL, which cannot be ignored,
and needs no change. A new regression test spawns the real runner with a
simulated launcher leak (preexec first ignoring the fatal signals) and
asserts the runner still exits 128+signal for each parametrized signal
within the normal budget. No production code, no runner script, no fixture
scripts, no assertion semantics change.

## Acceptance

The new regression test passes. The full trio passes 40/40 (37 prior plus the three new regression
parametrizations) under the exact former failure conditions: nohup launch,
four pytest workers, and deliberate CPU saturation, plus the existing solo
and foreground-lane passes. Runner
cleanup semantics (exit codes, fixture removal, record removal, port
release) are unchanged.

## Dependencies

Ticket 1190's sixty-second waits stay; they cover genuine teardown latency.

## Complexity

- Contract score: 1 (one frozen rule: reset fatal dispositions at spawn)
- State and timing score: 0 (test-harness spawn semantics only)
- Reach score: 0 (three test files, one shared helper each)
- Proof score: 1 (mechanism experimentally pinned; red/green under induced
  leak)
- Cost of error score: 0 (test-only; a wrong reset cannot weaken cleanup
  assertions)
- Total: 2
- Minimum level floor: none
- Final level: 2
- Reasons: mechanical preexec reset with a deterministic leak simulation
- Selected model: gpt-5.6-luna, high reasoning (level 2 implementer)

## Review

- Implementation evidence 2026-09-14: red first — the regression test with
  the leak simulated and no reset failed all three parametrizations in 91 s
  (deterministic on demand); green after the preexec reset — 3/3 in 5.1 s.
  Acceptance: 40/40 in 33-35 s under nohup plus four workers plus twelve CPU
  spinners; 40/40 solo in 42 s; 40/40 foreground lane under the same load
  that previously failed every [1] parametrization. A third signal-dependent
  site (the parallel-workers terminate test) was converted too; two
  non-signaled completion sites were deliberately left untouched.
- Design review: pending
- Code review: pending
