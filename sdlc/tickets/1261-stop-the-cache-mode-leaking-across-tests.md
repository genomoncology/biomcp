# Stop the cache mode leaking across tests

From `sdlc/issues/2026-09-27-cache-mode-is-fixed-for-the-whole-test-process.md`
(Ian's filing, tracking GitHub #286). Close #286 when this ships.

## Problem

`env_cache_mode()` (`src/sources/mod.rs:394`) stores the first
`BIOMCP_CACHE_MODE` value in a process-wide `OnceLock`. Tests that
set the variable to `off` and restore it afterwards
(`src/cli/discover.rs:293`,
`src/entities/drug/test_support.rs:149`) leave the OnceLock holding
`off` for the rest of the binary — order-dependent failures on
0.9.0 (8 failed single-threaded; each passes alone). `cache_infinite`
at `:442` already reads the variable per call, so the two readers
disagree about when a change takes effect.

## Fix

Option 2 from the issue, keeping release behavior unchanged: the
process keeps its once-read; tests get a scoped override (a
test-only seam set and cleared by a guard, like the existing
`NO_CACHE` task-local) that `env_cache_mode()` consults before the
OnceLock. The two existing setters convert to the guard so they
cannot leak. While there, make the two readers agree on one
function so a future mode value cannot split them again.

## Review

- Design review: n/a (Ian's issue specified option 2)
- Code review: REJECT on two serious findings (unserialized unit test on the
  shared override slot; stale line pin), fixed and verified 2026-09-27
- Verification: yellow gate at 8c37f045 — lint, test, spec, stress
  OK, zero failed lines

## Acceptance

- A regression test sets the mode to `off`, lets the guard drop,
  then runs a cache read in the same binary — the read hits the
  cache (the issue's success criterion).
- `cargo test --lib` passes with and without `--test-threads=1` on
  the gate host (the gate's default and the stress lane cover both
  shapes).
- The override seam exists only for tests; release builds have no
  new env read (assert or pin by test).
- Yellow gate green.
