# BioMCP plan

This file holds the 1.0 destination, the order of work, and each milestone's exit. It never says what is done. Read status from `pm daily`, `pm next`, `pm item` and `pm inbox`.

Rulings: [decisions](decisions/).

**Scope.** This plan covers the 0.9.2 release on `main` and the 1.0 line. The 0.9.2 release comes first. Its work lands on `main` only and never touches the 1.0 branch.

## Outcome

BioMCP 1.0 answers from shared typed values rather than from shapes it defines itself. Every command returns a type the shared library owns, so a caller gets the same value whichever surface it asks through, and a parsing fix reaches every consumer at once.

Today most commands return shapes defined here, and a handful return shared types. The gap is the work.

## How the two lines relate

`main` is the public download surface. Someone installing BioMCP gets it. Ian ruled on October 9 that [the release line flows into 1.0, never the reverse](decisions/2026-10-09-the-release-line-flows-into-1-0.md).

So the 1.0 branch merges `main` in continuously and never pushes code to it. Only records go to `main` from this lane: 1.0 tickets live in `main`'s ticket folder under the 1.0 milestone, so one database serves both lines and numbers cannot collide.

## Milestones

**1. Zero drift.** The 1.0 line carries the release line's tip.

Exit: the branch is never more than a day behind `main`, and a check reports its distance rather than anyone measuring it by hand.

**2. A suite that runs.** The 1.0 line's own gate passes on the build host.

Exit: the gate command completes with a reported result. Today the suite stalls when run as threads in one process because tests mutate process-global state, so the per-process form is the only one that finishes, and nobody has recorded the outcome either way.

**3. Shared types at the surface.** A command's output shape is not defined here.

Exit: every command returns a shared typed value, the hand-written pattern matching over provider payloads is gone, and no exported record carries raw bytes where a shared decoder exists.

**4. One ticket database.** The two lines never fork their records again.

Exit: every 1.0 ticket lives on `main` with its milestone, a check refuses a ticket number that exists on both lines in different files, and `pm next --milestone 1.0` is the whole queue.

## Order

Open tickets in the order I want them taken. Status comes from pm, never from this list.

- 2083
- 2086
- 2075
- 2074
- 2071
- 2081
- 2079
- 2080
- 2076
- 2094
- 2095
- 2085
- 2089
- 2090
- 2091
- 2092
- 2084
- 2065
- 2066
- 2067
- 2068
- 2087
- 2088
- 2073
- 2072
- 2077
- 2078
- 2082
- 2041
- 2006
- 2005
- 2003
- 2004

0.9.3 runs in two lanes at once. The first lane makes building and testing fast; the second, from 2085 to 2088, makes the program fast and fixes product answers. The second lane's tickets share no claimed file with the first lane's, so they run beside it rather than after it. Ian ruled that program speed matters more than build speed, and on 2026-10-14 that tests are deleted boldly and the suites split ([decision](decisions/2026-10-14-delete-tests-boldly-and-split-the-suites.md)).

First lane. 2083 comes first because every later ticket adds a changelog bullet, and until 2083 changes the file's shape each landing conflicts with the last. 2086 comes next because every landing reruns the full gates on pm's review-record commit, so each later landing waits one CI run less once it lands. 2075 follows 2086 because every later landing waits on CI two or three times; it runs the lanes in parallel off one build and moves the all-features, Windows, stress and release-panic jobs to `main`. 2074 follows 2075 because it needs 2083's changelog check and shares `.github/workflows/ci.yml`, whose chain is 2086, 2075, 2074, 2076. Then the test cleanup, the main aim of the milestone: 2071 takes the slow cases off the Python lane; 2081 deletes every test that is not one of the four kinds, in small batches that each land on their own, and its batches avoid files the second lane claims; 2079 lands a small replay server and golden suite first, which 2080 needs to bring the spec pages to about 3 minutes and which 2081's last batch needs for `search all`; 2076 replaces the ratchet scripts with compiler lints and deletes their self-tests. 2094 measures the result, closes any gap to about 10 minutes of CI and 3 minutes of `make test`, and makes CI a ticket's one full check. 2095 then makes the runners fail any routine test over 10 seconds, once 2071 and 2081 have removed or fixed the slow ones. The `Makefile` chain is 2075, 2071, 2076, 2095; each ticket in a chain lands before the next starts.

Second lane. 2085 lands first among the drug tickets, and 2090 and then 2068 rebase onto its drug lookup file. 2089 and 2090 send at the same time requests that BioMCP now sends one after another, with unchanged output. 2091 follows both because it changes how fresh GWAS and openFDA answers are, on the same call paths. 2092 removes a start-up stall on large caches. The product fixes 2084, 2066, 2067 and 2087 share no code file with each other or with the first lane. 2088 lands after 2065, whose health files sit beside its own.

Then the dependency and dead-code work. 2073 lands after 2065 because they edit different parts of the same two MCP files. The `Cargo.toml` chain is 2076, 2072, 2077, 2078; 2076 deletes the size inventory 2072 used to edit. 2077 and 2078 both edit `deny.toml`, so they land one after the other, and 2078 lands after 2088 because both edit the FDA orphan source. 2077 changes code only the all-features build compiles, so its proof runs `make full-feature-check` before landing. 2082 measures the result against the baseline record and the decision's targets, with the run-time block alternating warm and cache-off runs. 0.9.3 exits when 2082 records the after numbers and the tag passes its first rehearsal. The plans behind these tickets are `sdlc/designs/2026-10-12-0-9-3-iteration-speed.md` and `sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md`.

Among the 1.0 tickets, 2041 leads because the 1.0 branch sits hundreds of commits behind the release line, and every other ticket on this list merges into a branch that cannot absorb `main`. 2006 follows because it is the check that stops the two lines colliding again, and milestone 4 cannot close without it. 2005 comes next because it removes the reason a developer bypasses the commit hook, which is how unchecked work reaches a branch. 2003 and 2004 are the remaining patient-facing capability work.
