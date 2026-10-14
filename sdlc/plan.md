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

- 2070
- 2083
- 2086
- 2073
- 2071
- 2072
- 2065
- 2066
- 2084
- 2085
- 2067
- 2068
- 2074
- 2075
- 2076
- 2077
- 2078
- 2079
- 2080
- 2081
- 2082
- 2041
- 2006
- 2005
- 2003
- 2004

2070 comes first because every later ticket builds and tests through it. 2083 follows it because every later ticket adds a changelog bullet, and until 2083 changes the file's shape each landing conflicts with the last. 2086 comes next because every landing reruns the full gates on pm's review-record commit, so each later landing waits one CI run less once it lands; it shares no file with the tickets around it. 2073 and 2071 can run side by side: both wait for 2070, and they share no file. 2072 follows 2070 because both edit `Cargo.toml`. 2065 lands after 2073 because they edit different parts of the same two MCP files. 2066, 2084 and 2085 are three separate product fixes that share no code file with anything above, so they can run alongside any of it. 2067 and 2068 are the remaining product fixes. 2074 to 2076 change CI and lint, one at a time because they share workflow files. 2077 and 2078 finish the dependency work. 2079 builds the replay server and golden suite that 2080 and 2081 need. 2082 measures the result against the baseline record. 0.9.3 exits when 2082 records the after numbers and the tag passes its first rehearsal. The plans behind these tickets are `sdlc/designs/2026-10-12-0-9-3-iteration-speed.md` and `sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md`. Among the 1.0 tickets, 2041 leads because the 1.0 branch sits hundreds of commits behind the release line, and every other ticket on this list merges into a branch that cannot absorb `main`. 2006 follows because it is the check that stops the two lines colliding again, and milestone 4 cannot close without it. 2005 comes next because it removes the reason a developer bypasses the commit hook, which is how unchecked work reaches a branch. 2003 and 2004 are the remaining patient-facing capability work.
