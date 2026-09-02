---
flow: quickfix
priority: 10
---

# A gate test downloads live GTR data and times out under load, so no ticket can land

`tests/test_public_example_accessions.py::test_public_gtr_examples_resolve_against_live_gtr_bundle` fails in the factory's `before` stage. `make test` returns non-zero, `before` hands the ticket back as ready without consuming an attempt, and the ticket is claimed again. Nothing has landed on this channel since 2026-09-02T13:12Z. Ticket 1125 was claimed three times and spent about 76 minutes in `before` across those attempts, with `attempts consumed: 0` throughout.

The refund is correct behavior. A ticket that fails a gate through no fault of its own should not burn an attempt. The consequence is that the failure is invisible: the board shows a ticket that keeps looking ready, and only the before-evidence log says why.

```
FAILED tests/test_public_example_accessions.py::test_public_gtr_examples_resolve_against_live_gtr_bundle
  - subprocess.TimeoutExpired: Command '[... 'get', 'diagnostic', 'GTR000006692.3', 'regulatory']'
    timed out after 60 seconds
1 failed, 818 passed, 3 skipped in 551.20s
make[1]: *** [Makefile:49: test-contracts-prepared] Error 1
```

## Why this is a quickfix

`sdlc/project/before` documents the answer to a red main directly: exit 3 means "origin/main is red ... the channel backs off and a quickfix or the other run must resolve it before anything else can fly." The green-main gate skips for the quickfix flow, so this ticket can run while the gate it repairs is failing. A build ticket could not; it would fail the same test it exists to fix.

Priority 10 so it runs ahead of everything, because nothing else on this channel can land until it does.

## Cause

The test intends to run against a local bundle. `_prepare_public_gtr_bundle` at line 100 copies `GTR_FIXTURE_DIR` into a fresh temp directory and points `BIOMCP_GTR_DIR` at the copy. The copied bundle is old enough to read as stale, so the tool refreshes it from the network instead of using it. The first line of output is `Refreshing stale GTR data...`.

Measured against the repository build on 2026-09-02:

```
$ time biomcp get diagnostic GTR000006692.3 regulatory   # cold
73.5s wall
$ time biomcp get diagnostic GTR000006692.3 regulatory   # warm
33.5s wall
```

The test caps each command at 60 seconds, at line 157, and loops over more than six commands. The cold path is 73 seconds on an idle machine. In the factory the bundle is always cold, because the temp directory is new every run, and the machine is running several channels at once. So the cap is straddled rather than exceeded by a clear margin, which is why this passed for a while and now does not.

A gate test whose outcome depends on network speed and machine load is not a gate. It is a coin flip that stops the queue when it lands wrong.

## Required behavior

No test in the gate ladder reaches the network for data it already has a fixture for.

The prepared bundle is used as prepared. A test that hands the tool a fixture bundle gets that bundle's contents, not a refreshed copy of them.

## Done, observably

- `make test` passes with the network unavailable. Verified by running it with outbound traffic blocked, not by observing that it happened to pass.
- The test no longer prints `Refreshing stale GTR data...`, and a test pins that the prepared bundle is read without a refresh.
- The staleness refresh still works on the real path. A user with a genuinely stale bundle still gets fresh data, and a test pins that.
- No per-command timeout in this file is raised to accommodate a download. If a timeout changes, it changes because the work got smaller.
- The full suite is green.

## Boundary

Do not delete the test or mark it skipped. It covers the public accessions named in the tool's own help, and that coverage is the reason it exists.

Do not raise the 60-second cap as the fix. A larger number moves the coin flip rather than removing it.

Do not change what `get diagnostic` returns, or the GTR refresh behavior a real user sees.

Whether the fixture bundle should carry a version that never reads as stale, or whether the tool should honor an explicit "use this bundle as given" signal, is a design choice. Either satisfies this ticket. The second is the more honest one if a test ever needs to pin refresh behavior itself.
