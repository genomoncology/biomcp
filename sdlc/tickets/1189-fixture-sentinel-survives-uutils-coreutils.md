---
flow: build
priority: 1
deps: []
---

# 1189: The routine-fixture sentinel survives uutils coreutils

## Goal

The sentinel processes in `tests/test_routine_fixture_recovery.py` stay alive
for their full lifetime on hosts where `sleep` is the uutils multicall binary,
so the file passes on both GNU coreutils hosts and Ubuntu 25.10.

## Current Facts

Two spawn sites (lines 120 and 691) launch the sentinel as
`bash -c 'exec -a "$1" sleep 30' "fixture-owner" <owner_arg>`. The `exec -a`
override gives the sentinel a recognizable `argv[0]` marker in `/proc` while
running `sleep`. GNU coreutils ignores the overridden `argv[0]`; the uutils
multicall binary (Ubuntu 25.10, `sleep (uutils coreutils) 0.2.2`,
`/usr/lib/cargo/bin/coreutils/sleep`) validates `argv[0]` against the invoked
utility and exits 1 with "Security violation: Requested utility ... does not
match executable name". The sentinel therefore dies in about a tenth of a
second. Of the 34 failures in the full lane on the uutils host, 25 are
statically attributable (24 parametrizations of
test_cleanup_never_signals_an_unvalidated_ownership_record plus
test_runner_reaps_owned_lock_holder_before_acquiring_routine_lock); the
implementer must confirm the whole file passes rather than only the sentinel
tests. Reproduced 2026-09-12 on unmodified main at `193cfd6e`
on the 4-core gate host (1 failed in 0.13 s with `-x`), and in the full
`make test` pytest lane there (34 failed, 921 passed, all from this file).
The same file passes on the GNU-coreutils dev host.

## Scope

Change only the two sentinel spawn sites in
`tests/test_routine_fixture_recovery.py`. The replacement keeps the sentinel
process alive roughly 30 seconds, keeps the `argv[0]` marker visible in
`/proc/<pid>/cmdline` for whatever the ownership machinery reads, and works
identically on GNU and uutils coreutils. Implementation caveat from design
review: `exec -a "$1" bash -c 'sleep 30'` collapses via bash tail-call exec
into `sleep 30` and silently drops the marker; the inner script must not be
a single simple command (for example `'sleep 30 & wait'`, or a sleep loop).
No production code, no fixture
shell scripts, and no assertions change.

## Acceptance

The full `tests/test_routine_fixture_recovery.py` file passes on a GNU
coreutils host and on the uutils gate host. No other test changes behavior.

## Dependencies

None. Lands after ticket 1188 merges; stacked on 1187 in the meantime.

## Complexity

- Contract score: 0 (one exact existing rule: keep the sentinel alive with its marker)
- State and timing score: 0 (pure local test behavior; process lifetime only)
- Reach score: 0 (one test file)
- Proof score: 0 (one focused deterministic check on both coreutils flavors)
- Cost of error score: 0 (test-only, cheap local correction)
- Total: 0
- Minimum level floor: none
- Final level: 1
- Reasons: single-file test-independence fix
- Selected model: gpt-5.6-luna, high reasoning (level 1 implementer)

## Review

- Design review: ACCEPT 2026-09-12 with no blockers; findings incorporated:
  corrected failure attribution (24 parametrizations plus the runner test,
  whole-file pass required) and the bash tail-exec caveat in Scope. Marker
  consumers verified: routine-fixture-ownership.sh:107-111 (cmdline substring)
  and fixture-supervisor.py:80 (per-element fullmatch); nothing reads
  /proc/<pid>/comm.
- Code review: ACCEPT 2026-09-12 at commit 9b71d915; verified exactly two
  lines changed, marker preserved, no tail-exec collapse, ownership and
  supervisor classification unaffected at both sites, no unrelated riders.
  One remediation: the new spawn list line at tests line 691 exceeded ruff
  format's width and was the only new format divergence (the lines 362-377
  divergence pre-exists on main, confirmed by running ruff format --check on
  main); the line was split across lines in a follow-up commit. Full-file
  proof 61/61 on the GNU host; whole-file proof on the uutils gate host runs
  with the combined gates.
- Full gates (final): merged as PR #264. At the stack tip 38c56af2: lint OK
  and spec OK on the 16-core host, Rust 3451/3451 twice on the 4-core host;
  this file's lane failures on the uutils host dropped from 34 to 0. Earlier:
  pending on both hosts at the branch tip (combined stack with 1187)
