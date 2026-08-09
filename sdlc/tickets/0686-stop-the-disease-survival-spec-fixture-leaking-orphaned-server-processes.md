---
flow: build
priority: 9
---
# Stop the disease-survival spec fixture leaking orphaned server processes

Carried over from March ticket 686 when BioMCP moved to the sdlc
factory. The body below is March's, unchanged; it was already written to
stand alone. Work products from any earlier attempt:

    /home/ian/workspace/planning/biomcp/artifacts/686-stop-the-disease-survival-spec-fixture-leaking-orphaned-server-processes

March recorded this as failed with the reason 'unknown'.
That is an artefact of the queue being paused mid-flight, not a defeat:
no runner was alive to finish it. It starts clean here.
## Why this runs before the conversion tickets

Raised to priority 9 during the 2026-08-07 queue audit. This is not a cosmetic cleanup: the
leak actively breaks the tickets queued behind it.

- Ticket 666's design review recorded that `make spec` "stalled in later fixture cleanup and
  the 600-second harness timeout stopped it" — this cleanup path.
- The orphans held March's live-ownership signal after 666's fourth abort, so `march recover`
  refused to run until twelve processes were killed by hand.

Every remaining conversion ticket runs `make spec` repeatedly. Fixing this first removes a
known cause of stalls and blocked recoveries from all of them.

## Why

The disease-survival spec fixture leaks its HTTP server process on abnormal exit. I found
**twelve** orphaned servers on the workspace, all reparented to `systemd` (`ppid 1`):

| Origin | Age |
|---|---|
| `worktrees/678-repair-the-pmc-oa-package-url…` | 1d 9h (ticket merged) |
| `worktrees/666-convert-disease-phenotype…` | 5h 37m, plus 5 more from repeated runs |
| `/tmp/pytest-of-ian/pytest-903/test_routine_fixture_setup_doe1/workspace` | 3h 9m |

Command shape:

```
python3 - <workspace> <workspace>/.cache/spec-disease-survival.<rand>/base-url \
                      <workspace>/.cache/spec-disease-survival.<rand>/request.log
```

Each holds a loopback port and a `.cache/spec-disease-survival.*` directory for as long as
it lives. They accumulate: every aborted or timed-out spec run adds one or more, and they
survive worktree removal and ticket merge.

This is not cosmetic. Ticket 666's design review recorded that `make spec` "stalled in later
fixture cleanup and the 600-second harness timeout stopped it" — the same cleanup path. The
orphans also held March's live-ownership signal after 666's fourth abort, so
`march recover` refused to proceed until they were killed by hand.

The fixture already claims to reap stale processes: 666's design describes the runner-owned
fixture as one that "reaps stale processes." Whatever that reaper does, it did not collect
any of these twelve.

## Scope

1. Find where the disease-survival fixture starts its server and why the teardown path is
   skipped on abnormal exit — timeout, SIGINT, harness kill, or an exception between spawn
   and the cleanup handler.
2. Make teardown unconditional. The server must die when its owning run dies, including on
   SIGKILL of the parent. Prefer a mechanism that does not depend on the parent running
   cleanup code at all — a pipe/`PDEATHSIG`-style parent-death signal, or the child polling
   for its parent, rather than another `trap`.
3. Make the existing stale-process reaper actually collect orphans: match on the
   `spec-disease-survival.*` marker, verify `ppid == 1`, and reap on fixture startup so a
   fresh run cleans up after previous ones.
4. Remove the orphaned `.cache/spec-disease-survival.*` directories those processes left.
5. Check whether sibling spec fixtures share the same lifecycle helper and the same defect.
   `spec/fixtures/` has several; name each one checked and its result.

## Success Checklist

- [ ] Killing the parent run with `SIGKILL` mid-spec leaves **no** surviving
      `spec-disease-survival` process after a bounded grace period.
- [ ] A spec run that times out leaves no surviving fixture process.
- [ ] Starting the fixture reaps any pre-existing orphan matching the marker with `ppid 1`,
      and logs how many it collected.
- [ ] No `.cache/spec-disease-survival.*` directory outlives its run.
- [ ] A test asserts the above by spawning the fixture, killing the parent, and checking for
      survivors — not by inspecting cleanup code.
- [ ] Every other fixture in `spec/fixtures/` is audited and the result recorded, with the
      same fix applied wherever the defect is shared.
- [ ] `make lint`, `make test`, `make spec` green.

## Dependencies

None.

## Notes

Found while recovering ticket 666's fourth abort on 2026-08-06. I killed the twelve orphans
by hand to unblock `march recover`; the leak itself is untouched and will recur on the next
aborted spec run.

Do not fix this by adding a cleanup call to the abort path. The whole point is that the abort
path is exactly what does not run — a harness timeout or `SIGKILL` gives the parent no
opportunity to clean anything up. The child must be responsible for its own death.

## Operator note — 2026-08-07: the 01-design failure on record is not a real failure

This ticket shows `failed_step: 01-design`, reason `orphaned active ticket: no live runner
found`. That is bookkeeping from an operator mistake, not a defect in the ticket or in March.

The queue had been paused deliberately. I misread "go ahead and do the fixes" as authorization
to resume it, started the worker, and this ticket was picked up and reached 01-design (5/19
checks, ~12 minutes). When the error was caught the worker was stopped mid-step, which left
the ticket orphaned-active; `march doctor --fix` then auto-failed it.

Nothing here reflects on the work. When this ticket is next run, start it clean — the
preserved worktree state is from an interrupted run, not a considered one, and the recovery
plan already reports its evidence as incomplete.

## Also covered here: interruption leaves the lock held

March issue 611 recorded the same defect from the other direction and is
folded in rather than filed twice. Interrupting `make spec` left CTGov and
article fixture servers orphaned with PPID 1; the CTGov child kept an open
descriptor on `.cache/spec-routine-fixtures.lock`, so the next routine run
blocked at `flock` before reaching its own cleanup, and only manual killing
freed it.

The runner has cleanup traps; a parent interruption does not reach all
children. Fixing the leak without fixing this leaves the same stall one
Ctrl-C away.

Done when, for this part: each fixture server runs in a tracked process
group, startup reaps stale worktree-owned fixture children before taking
the routine lock, and a harness test interrupts a routine run and shows a
second run reaching the runner with no manual cleanup.

## Bound by the 2026-08-09 factory flight's design review

The first factory attempt's design was refused, correctly, and the
refusal's behaviorally-proven findings bind the next attempt:

- EXIT traps and local PID/env cleanup are NOT SIGKILL-safe and are
  not evidence a fixture is excluded. The refused design deferred
  `complexportal`, `drug-ae-fallback`, `mychem-empty`,
  `section-outcomes`, `study-download-error`, and `vaers` because they
  have local PID/env cleanup, and `article-federated-timeout` because
  it has a wrapper trap. The reviewer probed the complexportal path:
  started it under an owner shell with the real cleanup script on
  EXIT, SIGKILLed the shell, and both server and root survived. All
  seven need the same owner-death fix or behavioral proof of safety.
- The file-only fixtures (`cvx`, `ddinter`, `ema`, `gtr`, `study`,
  `who-ivd`, `who-pq`) start no server and are correctly excluded.
- The autonomous owner-death mechanism plus authenticated PPID-1
  marker recovery for the six routine ownership-helper fixtures was
  accepted in principle; keep it. The design must additionally state
  race-safe owner identity — pidfd with a validated fallback, not
  polling a reusable PID — and keep deletion restricted to canonical
  fixture roots.
- The review stage committed one repair on the old claim branch
  (1cc32958, loosening the stale-reaper log-wording assertion to
  count/kind/semantics). That branch's tip should be tagged before any
  teardown so the repair survives.

The six owner-death test cases and the marker-orphan case were run
and fail against current code — the red tests exist; the next design
starts from them.
